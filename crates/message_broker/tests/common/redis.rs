use std::sync::Arc;

use quests_message_broker::redis::Redis;

pub async fn build_redis(db_number: u8) -> Arc<Redis> {
    let url = format!("127.0.0.1:6379/{}", db_number);
    let redis = Redis::new(url.as_str())
        .await
        .expect("Can connect to redis");

    if !redis.ping().await {
        panic!("cannot ping redis");
    }
    Arc::new(redis)
}
