use actix_web::{dev::Server, App, HttpServer};

mod health;
mod routes;

pub async fn start_server(port: &str) -> Server {
    let server_address = format!("0.0.0.0:{port}");

    let server = HttpServer::new(|| App::new().configure(routes::services))
        .bind(&server_address)
        .unwrap()
        .run();

    log::info!("Quests System REST API running at http://{server_address}");

    server
}
