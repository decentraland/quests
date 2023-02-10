use std::sync::Arc;

use quests_message_broker::redis::Redis;

pub async fn build_redis() -> Arc<Redis> {
    let redis = Redis::new("127.0.0.1:6379")
        .await
        .expect("Can connect to redis");

    if !redis.ping().await {
        panic!("cannot ping redis");
    }

    Arc::new(redis)
}
