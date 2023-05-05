use quests_server::run_app;

#[actix_web::main]
async fn main() {
    log::info!("Starting Quests API...");

    run_app().await;
}
