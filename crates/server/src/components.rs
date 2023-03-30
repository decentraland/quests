use crate::configuration::Config;
use quests_db::{create_quests_db_component, Database};
use quests_message_broker::{
    events_queue::RedisEventsQueue, init_message_broker_components,
    quests_channel::RedisQuestsChannel,
};

pub async fn init_components() -> (Config, Database, RedisEventsQueue, RedisQuestsChannel) {
    let config = Config::new().expect("Unable to build up the config");

    log::debug!("Configuration: {config:?}");

    let quests_database = create_quests_db_component(&config.database_url)
        .await
        .expect("unable to run the migrations"); // we know that the migrations failed because if connection fails, the app panics

    let (events_queue, quests_channel) = init_message_broker_components(&config.redis_url).await;

    (config, quests_database, events_queue, quests_channel)
}
