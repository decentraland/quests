use crate::configuration::Config;
use dcl_rpc::stream_protocol::GeneratorYielder;
use quests_db::{create_quests_db_component, Database};
use quests_definitions::quests::UserUpdate;
use quests_message_broker::{
    events_queue::RedisMessagesQueue, init_message_broker_components_with_subscriber,
    quests_channel::RedisQuestsChannelSubscriber,
};

pub async fn init_components() -> (
    Config,
    Database,
    RedisMessagesQueue,
    RedisQuestsChannelSubscriber<GeneratorYielder<UserUpdate>>,
) {
    let config = Config::new().expect("Unable to build up the config");

    log::debug!("Configuration: {config:?}");

    let quests_database = create_quests_db_component(&config.database_url)
        .await
        .expect("unable to run the migrations"); // we know that the migrations failed because if connection fails, the app panics

    let (events_queue, quests_channel) = init_message_broker_components_with_subscriber::<
        GeneratorYielder<UserUpdate>,
    >(&config.redis_url)
    .await;

    quests_channel.listen(|users_update| {
        // // id => vec generators
        // let yielder = notifier.clone();
        // async move { yielder.r#yield(users_update).await.unwrap() }
    });

    (config, quests_database, events_queue, quests_channel)
}
