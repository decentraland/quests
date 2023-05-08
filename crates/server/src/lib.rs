pub mod api;
pub mod auth;
pub mod components;
pub mod configuration;
pub mod domain;
pub mod rpc;

use actix_web::web::Data;
use api::middlewares::initialize_telemetry;
use components::init_components;
use env_logger::init as initialize_logger;
use tokio::{select, signal};

pub async fn run_app() {
    initialize_logger();
    initialize_telemetry();

    let (config, db, redis_events_queue, redis_quests_channel) = init_components().await;

    // Need to be a Data type for Actix. When it's a Data type, it's an Arc too.
    // So this let us to reuse the Arc instead of having a Data<Arc<Arc<T>>>
    // and then we can do the `into_inner()` to get the Arc that Data created.
    let config_arc = Data::new(config);
    let db_arc = Data::new(db);
    let redis_events_queue_arc = Data::new(redis_events_queue);

    let actix_rest_api_server = api::run_server((
        config_arc.clone(),
        db_arc.clone(),
        redis_events_queue_arc.clone(),
    ))
    .await;

    let (warp_websocket_server, rpc_server) = rpc::run_rpc_server((
        config_arc.into_inner(),
        db_arc.into_inner(),
        redis_events_queue_arc.into_inner(),
        redis_quests_channel,
    ))
    .await;

    select! {
        _ = actix_rest_api_server => {},
        _ = warp_websocket_server => {},
        _ = rpc_server => {},
        _ = signal::ctrl_c() => {
            log::info!("> run_app > SIGINT catched. Exiting...");
        }
    }
}
