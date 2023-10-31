pub mod middlewares;
pub mod routes;

use self::routes::{
    errors::{json_extractor_config, path_extractor_config},
    query_extractor_config,
};
use crate::configuration::Config;
use actix_web::{
    body::MessageBody,
    dev::{Server, ServiceFactory},
    web::Data,
    App, HttpServer,
};
use dcl_http_prom_metrics::HttpMetricsCollector;
use quests_db::Database;
use quests_message_broker::messages_queue::RedisMessagesQueue;
use tracing_actix_web::TracingLogger;

pub async fn run_server(
    config: Data<Config>,
    database: Data<Database>,
    events_queue: Data<RedisMessagesQueue>,
    metrics_collector: Data<HttpMetricsCollector>,
) -> Server {
    let server_address = format!("0.0.0.0:{}", config.http_server_port);

    let server = HttpServer::new(move || {
        get_app_router(&config, &database, &events_queue, &metrics_collector)
    })
    .bind(&server_address)
    .unwrap() // Unwrap because if it's not able to bind, it doens't matter the panic
    .run();

    log::info!("Quests REST API running at http://{}", server_address);

    server
}

pub fn get_app_router(
    config: &Data<Config>,
    db: &Data<Database>,
    redis: &Data<RedisMessagesQueue>,
    metrics_collector: &Data<HttpMetricsCollector>,
) -> App<
    impl ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse<impl MessageBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let cors = actix_cors::Cors::permissive();
    App::new()
        .app_data(query_extractor_config())
        .app_data(json_extractor_config())
        .app_data(path_extractor_config())
        .app_data(config.clone())
        .app_data(db.clone())
        .app_data(redis.clone())
        .app_data(metrics_collector.clone())
        .wrap(middlewares::dcl_auth_middleware(
            [
                "POST:/api/quests",
                "DELETE:/api/quests/{quest_id}",
                "PUT:/api/quests/{quest_id}",
                "GET:/api/quests/{quest_id}/stats",
                "PUT:/api/quests/{quest_id}/activate",
            ],
            [
                "GET:/api/quests/{quest_id}",
                "GET:/api/quests/{quest_id}/reward",
                "GET:/api/creators/{user_address}/quests",
            ],
        ))
        .wrap(dcl_http_prom_metrics::metrics())
        .wrap(middlewares::metrics_token(&config.wkc_metrics_bearer_token))
        .wrap(TracingLogger::default())
        .wrap(cors)
        .configure(routes::services)
}
