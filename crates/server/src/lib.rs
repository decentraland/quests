pub mod api;
pub mod components;
pub mod configuration;
pub mod domain;
pub mod rpc;

use std::sync::Arc;

use api::middlewares::initialize_telemetry;
use components::init_components;
use env_logger::init as initialize_logger;
use tokio::{select, signal};

pub async fn run_app() {
    initialize_logger();
    initialize_telemetry();

    let (config, db, redis_events_queue, redis_quests_channel) = init_components().await;
    let (config, db, redis_events_queue) =
        (Arc::new(config), Arc::new(db), Arc::new(redis_events_queue));

    let (warp_websocket_server, rpc_server) = rpc::run_rpc_server((
        config.clone(),
        db.clone(),
        redis_events_queue.clone(),
        redis_quests_channel,
    ))
    .await;

    let actix_rest_api_server =
        api::run_server((config.into(), db.into(), redis_events_queue.into())).await;

    select! {
        _ = actix_rest_api_server => {
            log::info!("> run_app > REST API finished. Exiting...");
        },
        _ = warp_websocket_server => {
            log::info!("> run_app > Warp server finished. Exiting...");
        },
        _ = rpc_server => {
            log::info!("> run_app > RPC Server finished. Exiting...");
        },
        _ = signal::ctrl_c() => {
            log::info!("> run_app > SIGINT catched. Exiting...");
        }
    }
}
