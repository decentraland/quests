use crate::redis::Redis;
use async_trait::async_trait;
use deadpool_redis::{redis::AsyncCommands, Connection};
use futures_util::{Future, StreamExt as _};
use log::{debug, error};
use quests_protocol::{
    quests::{user_update, UserUpdate},
    ProtocolMessage,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[async_trait]
pub trait ChannelSubscriber: Send + Sync {
    type SubscriptionNotifier;
    async fn subscribe(
        &self,
        quest_instance_id: &str,
        subscription_notifier: Self::SubscriptionNotifier,
    );
    async fn unsubscribe(&self, quest_instance_id: &str);
}

#[async_trait]
pub trait ChannelPublisher<Publishment>: Send + Sync {
    async fn publish(&mut self, update: Publishment);
}

pub struct RedisChannelSubscriber<SubscriptionNotifier> {
    subscriptions: Arc<RwLock<HashMap<String, SubscriptionNotifier>>>,
    redis: Arc<Redis>,
    channel_name: String,
}

pub struct RedisChannelPublisher {
    publish: Connection,
    channel_name: String,
}

impl<SubscriptionNotifier> RedisChannelSubscriber<SubscriptionNotifier> {
    pub(crate) fn new(redis: Arc<Redis>, channel_name: &str) -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            redis,
            channel_name: channel_name.to_string(),
        }
    }
}

impl<SubscriptionNotifier: Send + Sync + 'static> RedisChannelSubscriber<SubscriptionNotifier> {
    /// Listen to new messages
    pub fn listen<U: Future<Output = ()> + Send + Sync>(
        &self,
        on_update_fn: impl Fn(&SubscriptionNotifier, UserUpdate) -> U + Send + Sync + 'static,
    ) {
        let subscriptions = self.subscriptions.clone();
        let redis = self.redis.clone(); // Should we have an Option to do an Option::take instead of clonning and leaving a useless and unused Arc instance?
        tokio::spawn(async move {
            let connection = redis
                .get_async_connection()
                .await
                .expect("to get a connection"); // TODO: Error handling

            let connection = deadpool_redis::Connection::take(connection);
            let mut pubsub = connection.into_pubsub();
            // TODO: channel_name
            let mut on_message_stream = pubsub.on_message();

            loop {
                match on_message_stream.next().await {
                    Some(message) => {
                        let payload = message.get_payload::<Vec<u8>>();
                        match payload {
                            Ok(payload) => {
                                let update = UserUpdate::decode(&*payload);
                                match update {
                                    Ok(update) => {
                                        if let Some(user_update::Message::QuestState(
                                            quest_state_update,
                                        )) = &update.message
                                        {
                                            let subscriptions = subscriptions.read().await;
                                            let subscription_notifier = subscriptions
                                                .get(&quest_state_update.quest_instance_id);
                                            if let Some(notifier) = subscription_notifier {
                                                on_update_fn(notifier, update).await;
                                            }
                                        }
                                    }
                                    Err(_) => error!("Couldn't deserialize quest update"),
                                }
                            }
                            Err(_) => error!("Couldn't retrieve payload"),
                        }
                    }
                    None => debug!("Couldn't read a message from stream"),
                }
            }
        });
    }
}

impl RedisChannelPublisher {
    pub async fn new(redis: Arc<Redis>, channel_name: &str) -> Self {
        let publish = redis
            .get_async_connection()
            .await
            .expect("to get a connection");

        Self {
            publish,
            channel_name: channel_name.to_string(),
        }
    }
}

#[async_trait]
impl<Notifier: Send + Sync> ChannelSubscriber for RedisChannelSubscriber<Notifier> {
    type SubscriptionNotifier = Notifier;
    async fn subscribe(
        &self,
        quest_instance_id: &str,
        subscription_notifier: Self::SubscriptionNotifier,
    ) {
        self.subscriptions
            .write()
            .await
            .insert(quest_instance_id.to_string(), subscription_notifier);
    }

    async fn unsubscribe(&self, quest_instance_id: &str) {
        self.subscriptions.write().await.remove(quest_instance_id);
    }
}

#[async_trait]
impl<Publishment: ProtocolMessage + 'static> ChannelPublisher<Publishment>
    for RedisChannelPublisher
{
    async fn publish(&mut self, publishment: Publishment) {
        let publishment_bin = publishment.encode_to_vec();
        self.publish
            .publish::<&str, Vec<u8>, String>(&self.channel_name, publishment_bin);
    }
}
