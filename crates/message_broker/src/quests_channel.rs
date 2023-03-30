use async_trait::async_trait;
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::Connection;
use futures_util::StreamExt as _;
use log::debug;
use log::error;
use quests_definitions::quests::{user_update, UserUpdate};
use quests_definitions::ProstMessage;
use std::future::Future;
use std::pin::Pin;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::redis::Redis;
type Response = Pin<Box<dyn Future<Output = ()> + Send + Sync>>;
pub type OnUpdate = Box<dyn Fn(UserUpdate) -> Response + Send + Sync>;

const QUESTS_CHANNEL_NAME: &str = "QUEST_UPDATES_CHANNEL";

#[async_trait]
pub trait QuestsChannel: Send + Sync {
    async fn subscribe(&self, quest_instance_id: &str, on_update: OnUpdate);
    async fn unsubscribe(&self, quest_instance_id: &str);
    async fn publish(&mut self, update: UserUpdate);
}

pub struct RedisQuestsChannel {
    publish: Connection,
    subscriptions: Arc<RwLock<HashMap<String, OnUpdate>>>,
}

impl RedisQuestsChannel {
    pub async fn new(redis: Arc<Redis>) -> Self {
        let publish = redis
            .get_async_connection()
            .await
            .expect("to get a connection");

        let subscriptions = Arc::new(RwLock::new(HashMap::default()));

        Self {
            publish,
            subscriptions,
        }
    }

    /// Listen to new messages
    pub async fn listen(&self, redis: Arc<Redis>) {
        let subscriptions = self.subscriptions.clone();
        tokio::spawn(async move {
            let connection = redis
                .get_async_connection()
                .await
                .expect("to get a connection"); // TODO: Error handling

            let connection = deadpool_redis::Connection::take(connection);
            let mut pubsub = connection.into_pubsub();
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
                                            let on_update_fn = subscriptions
                                                .get(&quest_state_update.quest_instance_id);
                                            if let Some(on_update) = on_update_fn {
                                                on_update(update).await;
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

#[async_trait]
impl QuestsChannel for RedisQuestsChannel {
    async fn subscribe(&self, quest_instance_id: &str, on_update: OnUpdate) {
        self.subscriptions
            .write()
            .await
            .insert(quest_instance_id.to_string(), on_update);
    }

    async fn unsubscribe(&self, quest_instance_id: &str) {
        self.subscriptions.write().await.remove(quest_instance_id);
    }

    async fn publish(&mut self, update: UserUpdate) {
        let update_bin = update.encode_to_vec();
        self.publish
            .publish::<&str, Vec<u8>, String>(QUESTS_CHANNEL_NAME, update_bin);
    }
}
