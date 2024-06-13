use crate::redis::Redis;
use deadpool_redis::redis::{aio::PubSub, AsyncCommands};
use futures_util::{Future, StreamExt as _};
use log::{debug, error};
use quests_protocol::definitions::*;
use std::sync::Arc;

#[derive(Debug)]
pub enum RedisChannelSubscriberError {
    RedisError,
    NoConnectionAvailable,
}

pub struct RedisChannelSubscriber {
    subscriptor: PubSub,
}

impl RedisChannelSubscriber {
    pub async fn new(
        redis: Arc<Redis>,
        channel_name: &str,
    ) -> Result<Self, RedisChannelSubscriberError> {
        let connection = if let Some(conn) = redis.get_async_connection().await {
            conn
        } else {
            return Err(RedisChannelSubscriberError::NoConnectionAvailable);
        };

        let connection = deadpool_redis::Connection::take(connection);
        let mut pubsub = connection.into_pubsub();

        pubsub
            .subscribe(channel_name)
            .await
            .map_err(|_| RedisChannelSubscriberError::RedisError)?;

        Ok(Self {
            subscriptor: pubsub,
        })
    }

    pub async fn on_new_message<
        NewPublishment: ProtocolMessage + Default,
        U: Future<Output = ()> + Send + Sync,
    >(
        &mut self,
        on_new_message_fn: impl Fn(NewPublishment) -> U + Send + Sync + 'static,
    ) {
        let mut on_message_stream = self.subscriptor.on_message();

        loop {
            if let Some(message) = on_message_stream.next().await {
                let payload = message.get_payload::<Vec<u8>>();
                match payload {
                    Ok(payload) => {
                        debug!("New message received from channel");
                        let update = NewPublishment::decode(&*payload);
                        match update {
                            Ok(update) => {
                                debug!("New publishment parsed {update:?}");
                                on_new_message_fn(update).await;
                            }
                            Err(_) => error!("Couldn't deserialize update"),
                        }
                    }
                    Err(_) => error!("Couldn't retrieve payload"),
                }
            }
        }
    }
}

pub struct RedisChannelPublisher {
    redis: Arc<Redis>,
    channel_name: String,
}

impl RedisChannelPublisher {
    pub fn new(redis: Arc<Redis>, channel_name: &str) -> Self {
        Self {
            redis,
            channel_name: channel_name.to_string(),
        }
    }

    pub async fn publish<P: ProtocolMessage + 'static>(&self, publishment: P) {
        debug!("Publish > Getting connection...");
        let mut publish = self
            .redis
            .get_async_connection()
            .await
            .expect("to get a connection"); // TODO: Handle error

        debug!("Publish > Encoding message...");
        let publishment_bin = publishment.encode_to_vec();

        debug!("Publish > Publishing...");
        let result: Result<usize, _> = publish.publish(&self.channel_name, publishment_bin).await;
        match result {
            Ok(result) => debug!("Publish > Done with response {result}"),
            Err(e) => error!("Couldn't publish message with error: {e:?}"),
        }
    }
}
