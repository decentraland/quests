use std::sync::Arc;

use async_trait::async_trait;
use quests_definitions::quests::*;

use crate::Redis;

#[async_trait]
pub trait EventsQueue {
    async fn push(&self, event: Event);
    async fn pop(&self) -> Event;
}

pub struct RedisEventsQueue {
    redis: Arc<Redis>,
}

impl<'a> RedisEventsQueue {
    pub fn new(redis: Arc<Redis>) -> Self {
        Self { redis }
    }
}

#[async_trait]
impl EventsQueue for RedisEventsQueue {
    async fn push(&self, event: Event) {
        let connection = self.redis.get_async_connection().await;

        todo!()
    }

    async fn pop(&self) -> Event {
        let connection = self.redis.get_async_connection().await;

        todo!()
    }
}
