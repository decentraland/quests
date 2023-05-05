use crate::configuration::Config;
use quests_db::{create_quests_db_component, Database};
use quests_message_broker::{
    channel::RedisChannelSubscriber, init_message_broker_components_with_subscriber,
    messages_queue::RedisMessagesQueue,
};

pub async fn init_components() -> (Config, Database, RedisMessagesQueue, RedisChannelSubscriber) {
    let config = Config::new().expect("Unable to build up the config");

    log::debug!("Configuration: {config:?}");

    let quests_database = create_quests_db_component(&config.database_url, true)
        .await
        .expect("unable to run the migrations"); // we know that the migrations failed because if connection fails, the app panics

    let (events_queue, quests_channel) =
        init_message_broker_components_with_subscriber(&config.redis_url).await;

    (config, quests_database, events_queue, quests_channel)
}
