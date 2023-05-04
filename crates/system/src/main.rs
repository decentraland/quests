use env_logger::init as initialize_logger;
use quests_system::{configuration::Config, run_app};

#[tokio::main]
async fn main() {
    let config = Config::new().expect("Can parse config");

    initialize_logger();

    run_app(&config).await;
}
