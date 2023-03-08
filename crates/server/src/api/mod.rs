pub mod middlewares;
pub mod routes;

use crate::configuration;

use self::middlewares::initialize_telemetry;
use self::routes::query_extractor_config;
use actix_web::{
    body::MessageBody,
    dev::{Server, ServiceFactory},
    web::Data,
    App, HttpServer,
};
use env_logger::init as initialize_logger;
use quests_db::{create_quests_db_component, Database};
use quests_message_broker::{create_events_queue, events_queue::RedisEventsQueue};
use tracing_actix_web::TracingLogger;

pub async fn run_server() -> Result<Server, std::io::Error> {
    initialize_logger();
    initialize_telemetry();

    let config = configuration::Config::new().expect("Unable to build up the config");
    let quests_database = create_quests_db_component(&config.database_url)
        .await
        .expect("unable to run the migrations"); // we know that the migrations failed because if connection fails, the app panics

    // Create events queue
    let events_queue = create_events_queue(&config.redis_url).await;

    let server_address = format!("0.0.0.0:{}", config.server_port);

    let config_app_data = Data::new(config);
    let quests_database_app_data = Data::new(quests_database);
    let quests_events_queue_app_adata = Data::new(events_queue);

    let server = HttpServer::new(move || {
        get_app_router(
            &config_app_data,
            &quests_database_app_data,
            &quests_events_queue_app_adata,
        )
    })
    .bind(&server_address)?
    .run();

    log::info!("Quests API running at http://{}", server_address);

    Ok(server)
}

pub fn get_app_router(
    config: &Data<configuration::Config>,
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
