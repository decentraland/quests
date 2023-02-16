use async_trait::async_trait;
use deadpool_redis::redis::aio::PubSub;
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::Connection;
use futures_util::StreamExt as _;
use log::debug;
use log::error;
use log::info;
use quests_definitions::quest_state::QuestState;
use serde::Deserialize;
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::redis::Redis;

#[derive(Serialize, Deserialize)]
pub struct QuestUpdate {
    pub state: QuestState,
} // TODO: move to definitions
pub type OnUpdate = Box<dyn Fn(QuestUpdate) + Send + Sync>;

#[async_trait]
pub trait QuestsChannel: Send + Sync {
    async fn subscribe(&mut self, quest_id: &str, on_update: OnUpdate);
    async fn unsubscribe(&mut self, quest_id: &str);
    async fn publish(&mut self, quest_id: &str, update: QuestUpdate);
}

pub struct RedisQuestsChannel {
    pubsub: PubSub,
    publish: Connection,
    subscriptions: Arc<RwLock<HashMap<String, OnUpdate>>>,
}

impl RedisQuestsChannel {
    pub async fn new(redis: Arc<Redis>) -> Self {
        let connection = redis
            .get_async_connection()
            .await
            .expect("to get a connection"); // TODO: Error handling

        let connection = deadpool_redis::Connection::take(connection);
        let pubsub = connection.into_pubsub();

        let publish = redis
            .get_async_connection()
            .await
            .expect("to get a connection");

        let subscriptions = Arc::new(RwLock::new(HashMap::default()));

        Self {
            publish,
            pubsub,
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
                                let update = bincode::deserialize::<QuestUpdate>(&payload);
                                match update {
                                    Ok(_update) => {
                                        let _subscriptions = subscriptions.read().await;
                                        todo!()
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
    async fn subscribe(&mut self, quest_id: &str, on_update: OnUpdate) {
        let subscription = self.pubsub.subscribe(quest_id).await;
        match subscription {
            Ok(_) => {
                self.subscriptions
                    .write()
                    .await
                    .insert(quest_id.to_string(), on_update);
            }
            Err(_) => error!("Couldn't subscribe to channel {}", quest_id),
        }
    }

    async fn unsubscribe(&mut self, quest_id: &str) {
        match self.pubsub.unsubscribe(quest_id).await {
            Ok(_) => {
                self.subscriptions.write().await.remove(quest_id);
            }
            Err(_) => info!("Couldn't unsubscribe to {}", quest_id),
        };
    }

    async fn publish(&mut self, quest_id: &str, update: QuestUpdate) {
        let update_bin = bincode::serialize(&update).expect("can serialize update"); // TODO: error handling
        self.publish
            .publish::<&str, Vec<u8>, String>(quest_id, update_bin);
    }
}
