use quests_server::{init_components, run_server};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log::info!("Starting Quests API...");

    let app_components = init_components(None).await;

    run_server(app_components)?.await
}
