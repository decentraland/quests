pub mod channel;
pub mod messages_queue;
pub mod redis;

use channel::{RedisChannelPublisher, RedisChannelSubscriber};
use messages_queue::RedisMessagesQueue;
use redis::Redis;
use std::sync::Arc;

pub async fn init_message_broker_components_with_subscriber(
    redis_url: &str,
) -> (RedisMessagesQueue, RedisChannelSubscriber) {
    let redis = init_redis(redis_url).await;

    let redis_events_queue = init_events_queue(redis.clone());
    let quests_channel = init_quests_channel_subscriber(redis);

    (redis_events_queue, quests_channel)
}

pub async fn init_message_broker_components_with_publisher(
    redis_url: &str,
) -> (RedisMessagesQueue, RedisChannelPublisher) {
    let redis = init_redis(redis_url).await;

    let redis_events_queue = init_events_queue(redis.clone());
    let quests_channel = init_quests_channel_publisher(redis).await;

    (redis_events_queue, quests_channel)
}

async fn init_redis(redis_url: &str) -> Arc<Redis> {
    log::info!("Redis URL: {}", &redis_url);
    let redis = Redis::new(redis_url).await.expect("Can connect to Redis");
    Arc::new(redis)
}

fn init_events_queue(redis: Arc<Redis>) -> RedisMessagesQueue {
    RedisMessagesQueue::new(redis, "events:queue")
}

pub const QUEST_UPDATES_CHANNEL_NAME: &str = "QUEST_UPDATES";

fn init_quests_channel_subscriber(redis: Arc<Redis>) -> RedisChannelSubscriber {
    RedisChannelSubscriber::new(redis)
}

async fn init_quests_channel_publisher(redis: Arc<Redis>) -> RedisChannelPublisher {
    RedisChannelPublisher::new(redis, QUEST_UPDATES_CHANNEL_NAME).await
}
