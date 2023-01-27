use std::sync::Arc;

use async_trait::async_trait;
use deadpool_redis::redis::aio::PubSub;

use crate::Redis;

pub enum QuestUpdate {}

pub type OnUpdate = Box<dyn Fn(QuestUpdate) + Send + Sync>;

#[async_trait]
pub trait QuestsChannel {
    async fn subscribe(&self, quest_id: &str, on_update: OnUpdate);
    async fn unsubscribe(&self, quest_id: &str);
    async fn publish(&self, quest_id: &str, update: QuestUpdate);
}

pub struct RedisQuestsChannel {
    pubsub: PubSub,
}

impl RedisQuestsChannel {
    pub async fn new(redis: Arc<Redis>) -> Self {
        let connection = redis
            .get_async_connection()
            .await
            .expect("to get a connection"); // TODO: Error handling

        let connection = deadpool_redis::Connection::take(connection);
        Self {
            pubsub: connection.into_pubsub(),
        }
    }
}

#[async_trait]
impl QuestsChannel for RedisQuestsChannel {
    async fn subscribe(&self, quest_id: &str, on_update: OnUpdate) {
        todo!()
    }

    async fn unsubscribe(&self, quest_id: &str) {
        todo!()
    }

    async fn publish(&self, quest_id: &str, update: QuestUpdate) {
        todo!()
    }
}
