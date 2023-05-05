use configuration::Config;
use tokio::{select, signal};

pub mod api;
pub mod configuration;
pub mod event_processing;

pub async fn run_app(config: &Config) {
    let server = api::start_server(&config.http_server_port).await;
    let event_processing = event_processing::start_event_processing(config).await;

    select! {
        _ = server => {},
        result = event_processing => {
            match result {
                Ok(_) => {},
                Err(e) => log::debug!("> run_app > Event processing failed due {e:?}"),
            }
            log::info!("> run_app > Event processing finished");
        },
        _ = signal::ctrl_c() => {
            log::info!("> run_app > SIGINT catched. Exiting...");
        }
    }
}
