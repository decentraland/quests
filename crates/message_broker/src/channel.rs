use crate::redis::Redis;
use async_trait::async_trait;
use deadpool_redis::{redis::AsyncCommands, Connection};
use futures_util::{Future, StreamExt as _};
use log::{debug, error};
use quests_protocol::ProtocolMessage;
use std::sync::Arc;

pub trait ChannelSubscriber: Send + Sync {
    fn subscribe<NewPublishment: ProtocolMessage + Default, U: Future<Output = ()> + Send + Sync>(
        &self,
        channel_name: &str,
        on_update_fn: impl Fn(NewPublishment) -> U + Send + Sync + 'static,
    );
}

#[async_trait]
pub trait ChannelPublisher<Publishment>: Send + Sync {
    async fn publish(&mut self, update: Publishment);
}

pub struct RedisChannelSubscriber {
    redis: Arc<Redis>,
}

impl RedisChannelSubscriber {
    pub(crate) fn new(redis: Arc<Redis>) -> Self {
        Self { redis }
    }
}

impl ChannelSubscriber for RedisChannelSubscriber {
    /// Listens to a specific channel for new messages
    fn subscribe<
        NewPublishment: ProtocolMessage + Default,
        U: Future<Output = ()> + Send + Sync,
    >(
        &self,
        channel_name: &str,
        on_update_fn: impl Fn(NewPublishment) -> U + Send + Sync + 'static,
    ) {
        let redis = self.redis.clone(); // Should we have an Option to do an Option::take instead of clonning and leaving a useless and unused Arc instance?
        let channel_name = channel_name.to_string();
        tokio::spawn(async move {
            let connection = redis
                .get_async_connection()
                .await
                .expect("to get a connection"); // TODO: Error handling

            let connection = deadpool_redis::Connection::take(connection);
            let mut pubsub = connection.into_pubsub();
            pubsub
                .subscribe(channel_name)
                .await
                .expect("to be able to listen to this channel");
            let mut on_message_stream = pubsub.on_message();

            loop {
                match on_message_stream.next().await {
                    Some(message) => {
                        let payload = message.get_payload::<Vec<u8>>();
                        match payload {
                            Ok(payload) => {
                                let update = NewPublishment::decode(&*payload);
                                match update {
                                    Ok(update) => {
                                        on_update_fn(update).await;
                                    }
                                    Err(_) => error!("Couldn't deserialize update"),
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

pub struct RedisChannelPublisher {
    publish: Connection,
    channel_name: String,
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
impl<Publishment: ProtocolMessage + 'static> ChannelPublisher<Publishment>
    for RedisChannelPublisher
{
    async fn publish(&mut self, publishment: Publishment) {
        let publishment_bin = publishment.encode_to_vec();
        self.publish
            .publish::<&str, Vec<u8>, String>(&self.channel_name, publishment_bin);
    }
}
