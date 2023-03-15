pub mod middlewares;
pub mod routes;

use self::middlewares::initialize_telemetry;
use self::routes::query_extractor_config;
use crate::{components::init_components, configuration::Config};
use actix_web::{
    body::MessageBody,
    dev::{Server, ServiceFactory},
    web::Data,
    App, HttpServer,
};
use env_logger::init as initialize_logger;
use quests_db::Database;
use quests_message_broker::events_queue::RedisEventsQueue;
use tracing_actix_web::TracingLogger;

pub async fn run_server() -> Result<Server, std::io::Error> {
    initialize_logger();
    initialize_telemetry();

    let components = init_components().await;

    let server_address = format!("0.0.0.0:{}", components.0.server_port);

    let config = Data::new(components.0);
    let db = Data::new(components.1);
    let redis_events_queue = Data::new(components.2);

    let config_moved = config.clone();
    let db_moved = db.clone();
    let redis_events_queue_moved = redis_events_queue.clone();

    let server = HttpServer::new(move || {
        get_app_router(&config_moved, &db_moved, &redis_events_queue_moved)
    })
    .bind(&server_address)?
    .run();

    // Take Arc inside of the Data for the RPC Server
    let config = config.into_inner();
    let db = db.into_inner();
    let redis_events_queue = redis_events_queue.into_inner();

    log::info!("Quests API running at http://{}", server_address);

    Ok(server)
}

pub fn get_app_router(
    config: &Data<Config>,
    db: &Data<Database>,
    redis: &Data<RedisEventsQueue>,
) -> App<
    impl ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse<impl MessageBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .app_data(query_extractor_config())
        .app_data(config.clone())
        .app_data(db.clone())
        .app_data(redis.clone())
        .wrap(middlewares::metrics())
        .wrap(TracingLogger::default())
        .wrap(middlewares::metrics_token(&config.wkc_metrics_bearer_token))
        .configure(routes::services)
}
