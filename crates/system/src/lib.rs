mod configuration;
mod event_processing;

use configuration::Config;
use event_processing::process_event;
use quests_db::create_quests_db_component;
use quests_message_broker::events_queue::{EventsQueue, RedisEventsQueue};
use quests_message_broker::quests_channel::RedisQuestsChannel;
use quests_message_broker::redis::Redis;
use std::sync::Arc;

pub type Error = String;
pub type EventProcessingResult<T> = Result<T, Error>;

/// Starts the main processing task which reads events from the queue, updates the quest states and
/// publishes the changes.
///
/// Panics if can't parse the config
pub async fn start_event_processing() -> EventProcessingResult<()> {
    // TODO: read from config
    let config = Config::new().expect("Can parse config");

    // Create Redis pool
    let redis = Redis::new(&config.redis_url)
        .await
        .expect("Can connect to Redis");
    let redis = Arc::new(redis);

    // Create events queue
    let events_queue = RedisEventsQueue::new(redis.clone());
    let events_queue = Arc::new(events_queue);

    // Create quests channel
    let quests_channel = RedisQuestsChannel::new(redis.clone()).await;
    let quests_channel = Arc::new(quests_channel);

    // Create DB
    let database = create_quests_db_component(&config.db_url)
        .await
        .map_err(|_| "Couldn't connect to the database".to_string())?;
    let database = Arc::new(database);

    loop {
        // Read items from events queue
        let event = events_queue.pop().await;
        // - Spawn task to process the event
        tokio::spawn(process_event(
            event,
            quests_channel.clone(),
            database.clone(),
            events_queue.clone(),
        ));
    }
}
