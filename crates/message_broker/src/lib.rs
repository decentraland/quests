use std::sync::Arc;

use events_queue::RedisEventsQueue;
use redis::Redis;

pub mod events_queue;
pub mod quests_channel;
pub mod redis;

pub async fn create_events_queue(redis_url: &str) -> RedisEventsQueue {
    let redis = Redis::new(redis_url).await.expect("Can connect to Redis");
    let redis = Arc::new(redis);

    RedisEventsQueue::new(redis)
}
