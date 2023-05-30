use crate::redis::Redis;
use async_trait::async_trait;
use deadpool_redis::redis::AsyncCommands;
use quests_protocol::definitions::ProtocolMessage;
use std::sync::Arc;

pub type MessagesQueueResult<T> = Result<T, String>;

#[async_trait]
pub trait MessagesQueue<T>: Send + Sync {
    async fn push(&self, item: &T) -> MessagesQueueResult<usize>;
    async fn pop(&self) -> MessagesQueueResult<T>;
}

pub struct RedisMessagesQueue {
    redis: Arc<Redis>,
    queue_name: String,
}

impl RedisMessagesQueue {
    pub fn new(redis: Arc<Redis>, channel_name: &str) -> Self {
        Self {
            redis,
            queue_name: channel_name.to_string(),
        }
    }
}

#[async_trait]
impl<T: ProtocolMessage + Default> MessagesQueue<T> for RedisMessagesQueue {
    async fn push(&self, message: &T) -> MessagesQueueResult<usize> {
        let mut connection = self
            .redis
            .get_async_connection()
            .await
            .ok_or("Failed to get a connection")?;
        let message = message.encode_to_vec();
        let queue_size: usize = connection
            .rpush(&self.queue_name, message)
            .await
            .map_err(|err| format!("Failed to send push command: {err}"))?;

        Ok(queue_size)
    }

    async fn pop(&self) -> MessagesQueueResult<T> {
        let mut connection = self
            .redis
            .get_async_connection()
            .await
            .ok_or("Failed to get a connection")?;

        // it returns an array of [key = {channel_name}, value = message]
        let result: Vec<Vec<u8>> = connection
            .blpop(&self.queue_name, 0)
            .await
            .map_err(|err| format!("Couldn't get an element from the messages queue: {err}"))?;

        T::decode(&*result[1])
            .map_err(|_| "Couldn't deserialize response as an return type".to_string())
    }
}
