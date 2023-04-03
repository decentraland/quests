use crate::redis::Redis;
use async_trait::async_trait;
use deadpool_redis::redis::AsyncCommands;
use quests_definitions::ProstMessage;
use std::sync::Arc;

pub type EventsQueueResult<T> = Result<T, String>;

#[async_trait]
pub trait EventsQueue<T>: Send + Sync {
    async fn push(&self, item: &T) -> EventsQueueResult<usize>;
    async fn pop(&self) -> EventsQueueResult<T>;
}

pub struct RedisEventsQueue {
    redis: Arc<Redis>,
}

impl RedisEventsQueue {
    pub fn new(redis: Arc<Redis>) -> Self {
        Self { redis }
    }
}

const EVENTS_QUEUE: &str = "events:queue";

#[async_trait]
impl<T: ProstMessage + Default> EventsQueue<T> for RedisEventsQueue {
    async fn push(&self, event: &T) -> EventsQueueResult<usize> {
        let mut connection = self
            .redis
            .get_async_connection()
            .await
            .ok_or("Failed to get a connection")?;
        let event = event.encode_to_vec();
        let queue_size: usize = connection
            .rpush(EVENTS_QUEUE, event)
            .await
            .map_err(|err| format!("Failed to send push command: {err}"))?;

        Ok(queue_size)
    }

    async fn pop(&self) -> EventsQueueResult<T> {
        let mut connection = self
            .redis
            .get_async_connection()
            .await
            .ok_or("Failed to get a connection")?;

        // it returns an array of [key = "events:queue", value = event]
        let result: Vec<Vec<u8>> = connection
            .blpop(EVENTS_QUEUE, 0)
            .await
            .map_err(|err| format!("Couldn't get an element from the events queue: {err}"))?;

        let item =
            T::decode(&*result[1]).map_err(|_| "Couldn't deserialize response as an Event")?;

        Ok(item)
    }
}
