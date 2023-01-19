use actix_web::{dev::Server, App, HttpServer};
use env_logger::init as initialize_logger;
use tracing_actix_web::TracingLogger;

mod components;
mod middlewares;
mod routes;

use crate::middlewares::init_telemetry;

pub fn run_server() -> Result<Server, std::io::Error> {
    initialize_logger();
    init_telemetry();

    let config =
        components::Config::new().expect("unable to build up the Configuration for the App");

    log::info!("App Config: {config:?}");

    let server_address = format!("{}:{}", config.server.host, config.server.port);
    let bearer_token = config.wkc_metrics_bearer_token.clone();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middlewares::metrics())
            .wrap(TracingLogger::default())
            .wrap(middlewares::metrics_token(&bearer_token))
            .configure(routes::services)
    })
    .bind(&server_address)?
    .run();

    log::info!("Quests API running at http://{}", server_address);

    Ok(server)
}
