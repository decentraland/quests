pub mod events_queue;
pub mod quests_channel;
pub mod redis;

use events_queue::RedisEventsQueue;
use quests_channel::RedisQuestsChannel;
use redis::Redis;
use std::sync::Arc;

pub async fn init_message_broker_components(
    redis_url: &str,
) -> (RedisEventsQueue, RedisQuestsChannel) {
    log::info!("Redis URL: {}", &redis_url);
    let redis = Redis::new(redis_url).await.expect("Can connect to Redis");
    let redis = Arc::new(redis);

    let redis_events_queue = RedisEventsQueue::new(redis.clone());
    let quests_channel = RedisQuestsChannel::new(redis.clone()).await;

    quests_channel.listen(redis);

    (redis_events_queue, quests_channel)
}
