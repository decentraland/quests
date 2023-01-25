use actix_web::{
    body::MessageBody,
    dev::{Server, ServiceFactory},
    web::Data,
    App, HttpServer,
};
use env_logger::init as initialize_logger;
use quests_db::{create_quests_db_component, Database};
use tracing_actix_web::TracingLogger;

pub mod configuration;
mod middlewares;
pub mod routes;

use crate::routes::query_extractor_config;

use crate::middlewares::init_telemetry;

pub async fn run_server() -> Result<Server, std::io::Error> {
    initialize_logger();
    init_telemetry();

    let config = configuration::Config::new().expect("Unable to build up the config");
    let quests_database = create_quests_db_component(&config.database_url.as_str())
        .await
        .expect("unable to run the migrations"); // we know that the migrations failed because if connection fails, the app panics

    log::info!("App Config:  {:?}", config);

    let server_address = format!("0.0.0.0:{}", config.server_port);

    let config_app_data = Data::new(config);
    let quests_database_app_data = Data::new(quests_database);

    let server =
        HttpServer::new(move || get_app_router(&config_app_data, &quests_database_app_data))
            .bind(&server_address)?
            .run();

    log::info!("Quests API running at http://{}", server_address);

    Ok(server)
}

pub fn get_app_router(
    config: &Data<configuration::Config>,
    db: &Data<Database>,
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
        .wrap(middlewares::metrics())
        .wrap(TracingLogger::default())
        .wrap(middlewares::metrics_token(&config.wkc_metrics_bearer_token))
        .configure(routes::services)
}
