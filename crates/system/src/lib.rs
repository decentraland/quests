use configuration::Config;
use tokio::{select, signal};

pub mod api;
pub mod configuration;
pub mod event_processing;
pub mod quests;
pub use quests::*;
mod rewards;

pub const QUESTS_EVENTS_QUEUE_NAME: &str = "events:queue";
pub const QUESTS_CHANNEL_NAME: &str = "QUEST_UPDATES";

pub async fn run_app(config: &Config) {
    let server = api::start_server(&config.http_server_port).await;
    let Ok(event_processor) = event_processing::EventProcessor::from_config(config).await else {
        return;
    };

    let event_processing = event_processing::start_event_processing(event_processor);

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
