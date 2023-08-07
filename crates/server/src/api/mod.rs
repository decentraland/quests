pub mod middlewares;
pub mod routes;

use self::routes::query_extractor_config;
use crate::configuration::Config;
use actix_web::{
    body::MessageBody,
    dev::{Server, ServiceFactory},
    web::Data,
    App, HttpServer,
};
use quests_db::Database;
use quests_message_broker::messages_queue::RedisMessagesQueue;
use tracing_actix_web::TracingLogger;

pub async fn run_server(
    config: Data<Config>,
    database: Data<Database>,
    events_queue: Data<RedisMessagesQueue>,
) -> Server {
    let server_address = format!("0.0.0.0:{}", config.http_server_port);

    let server = HttpServer::new(move || get_app_router(&config, &database, &events_queue))
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
        .app_data(config.clone())
        .app_data(db.clone())
        .app_data(redis.clone())
        .wrap(cors)
        .wrap(middlewares::metrics())
        .wrap(TracingLogger::default())
        .wrap(middlewares::metrics_token(&config.wkc_metrics_bearer_token))
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
        .configure(routes::services)
}
