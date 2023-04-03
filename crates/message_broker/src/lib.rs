pub mod events_queue;
pub mod quests_channel;
pub mod redis;

use events_queue::RedisEventsQueue;
use quests_channel::{RedisQuestsChannelPublisher, RedisQuestsChannelSubscriber};
use redis::Redis;
use std::sync::Arc;

pub async fn init_message_broker_components_with_subscriber<F>(
    redis_url: &str,
) -> (RedisEventsQueue, RedisQuestsChannelSubscriber<F>) {
    let redis = init_redis(redis_url).await;

    let redis_events_queue = RedisEventsQueue::new(redis.clone());
    let quests_channel = init_quests_channel_subscriber(redis);

    (redis_events_queue, quests_channel)
}

pub async fn init_message_broker_components_with_publisher(
    redis_url: &str,
) -> (RedisEventsQueue, RedisQuestsChannelPublisher) {
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

fn init_events_queue(redis: Arc<Redis>) -> RedisEventsQueue {
    RedisEventsQueue::new(redis)
}

fn init_quests_channel_subscriber<SubscriptionNotifier>(
    redis: Arc<Redis>,
) -> RedisQuestsChannelSubscriber<SubscriptionNotifier> {
    RedisQuestsChannelSubscriber::new(redis)
}

async fn init_quests_channel_publisher(redis: Arc<Redis>) -> RedisQuestsChannelPublisher {
    RedisQuestsChannelPublisher::new(redis).await
}
