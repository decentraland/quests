pub mod api;
pub mod configuration;
pub mod domain;
pub mod rpc;

use std::sync::Arc;

use api::middlewares::initialize_telemetry;
use dcl_http_prom_metrics::HttpMetricsCollectorBuilder;
use env_logger::init as initialize_logger;
use quests_db::create_quests_db_component;
use quests_message_broker::{
    channel::{RedisChannelPublisher, RedisChannelSubscriber},
    messages_queue::RedisMessagesQueue,
    redis::Redis,
};
use quests_system::{event_processing, QUESTS_CHANNEL_NAME, QUESTS_EVENTS_QUEUE_NAME};
use tokio::select;

use crate::configuration::Config;

pub async fn run_app() {
    initialize_logger();
    initialize_telemetry();

    let config = Config::new().expect("> run_app > invalid config");
    let config = Arc::new(config);

    let database = create_quests_db_component(&config.database_url, true)
        .await
        .expect("> run_app > unable to run the migrations");
    let database = Arc::new(database);

    let redis = Redis::new(&config.redis_url)
        .await
        .expect("> run_app > Couldn't initialize redis connection");
    let redis = Arc::new(redis);

    let events_queue = RedisMessagesQueue::new(redis.clone(), QUESTS_EVENTS_QUEUE_NAME);
    let events_queue = Arc::new(events_queue);

    let quests_channel_publisher = Arc::new(RedisChannelPublisher::new(
        redis.clone(),
        QUESTS_CHANNEL_NAME,
    ));
    let quests_channel_subscriber = RedisChannelSubscriber::new(redis.clone());

    let http_metrics_collector = Arc::new(HttpMetricsCollectorBuilder::default().build());

    let (warp_websocket_server, rpc_server) = rpc::run_rpc_server((
        config.clone(),
        database.clone(),
        events_queue.clone(),
        quests_channel_subscriber,
        quests_channel_publisher.clone(),
    ))
    .await;

    let event_processing = event_processing::run_event_processor(
        database.clone(),
        events_queue.clone(),
        quests_channel_publisher.clone(),
    );

    let actix_rest_api_server = api::run_server(
        config.into(),
        database.into(),
        events_queue.into(),
        http_metrics_collector.into(),
    )
    .await;

    select! {
        _ = tokio::signal::ctrl_c() => {
            log::info!("> run_app > SIGINT catched. Exiting...");
        },
        _ = actix_rest_api_server => {
            log::info!("> run_app > REST API finished. Exiting...");
        },
        _ = warp_websocket_server => {
            log::info!("> run_app > Warp server finished. Exiting...");
        },
        _ = rpc_server => {
            log::info!("> run_app > RPC Server finished. Exiting...");
        },
        _ = event_processing => {
            log::info!("> run_app > Event processing finished. Exiting...");
        }
    }
}
