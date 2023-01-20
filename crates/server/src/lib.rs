use actix_web::{dev::Server, web::Data, App, HttpServer};
use components::AppComponents;
use env_logger::init as initialize_logger;
use tracing_actix_web::TracingLogger;

mod components;
mod middlewares;
mod routes;

use crate::middlewares::init_telemetry;

pub fn run_server(app_components: AppComponents) -> Result<Server, std::io::Error> {
    initialize_logger();
    init_telemetry();

    log::info!("App Config:  {:?}", app_components.config);

    let server_address = format!(
        "{}:{}",
        app_components.config.server.host, app_components.config.server.port
    );
    let bearer_token = app_components.config.wkc_metrics_bearer_token.clone();

    let actix_app_data = Data::new(app_components);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(actix_app_data.clone()))
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

pub async fn init_components(custom_config: Option<components::Config>) -> AppComponents {
    AppComponents::new(custom_config).await
}
