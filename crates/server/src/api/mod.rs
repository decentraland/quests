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

    let (config, db, redis_events_queue) = init_components().await;

    let server_address = format!("0.0.0.0:{}", config.server_port);

    let config = Data::new(config);
    let db = Data::new(db);
    let redis_events_queue = Data::new(redis_events_queue);

    let server = HttpServer::new(move || get_app_router(&config, &db, &redis_events_queue))
        .bind(&server_address)?
        .run();

    // TODO: Take Arc inside of the Data for the RPC Server

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
