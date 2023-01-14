use actix_web::{dev::Server, App, HttpServer};
use env_logger::init as initialize_logger;

mod middlewares;
mod routes;

pub fn run_server() -> Result<Server, std::io::Error> {
    initialize_logger();

    // TODO: read from config
    let server_address = "[::1]:9000";
    let bearer_token = "token";

    let server = HttpServer::new(|| {
        App::new()
            .wrap(middlewares::metrics())
            .wrap(middlewares::telemetry())
            .wrap(middlewares::metrics_token(bearer_token))
            .configure(routes::services)
    })
    .bind(server_address)?
    .run();

    log::info!("Quests API running at http://{}", server_address);

    Ok(server)
}
