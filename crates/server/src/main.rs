use quests_server::api::run_server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log::info!("Starting Quests API...");
    run_server().await?.await
}
