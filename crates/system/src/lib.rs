pub mod configuration;
mod event_processing;

use configuration::Config;
use env_logger::init as initialize_logger;
use event_processing::{process_event, ProcessEventResult};
use log::{error, info};
use quests_db::{create_quests_db_component, Database};
use quests_message_broker::{
    channel::RedisChannelPublisher,
    init_message_broker_components_with_publisher,
    messages_queue::{MessagesQueue, RedisMessagesQueue},
};
use std::sync::Arc;
use tokio::task::JoinHandle;

pub type Error = String;
pub type EventProcessingResult<T> = Result<T, Error>;

pub struct EventProcessor {
    pub events_queue: Arc<RedisMessagesQueue>,
    quests_channel: Arc<RedisChannelPublisher>,
    database: Arc<Database>,
}

impl EventProcessor {
    pub async fn from_config(config: &Config) -> EventProcessingResult<Self> {
        let (events_queue, quests_channel) =
            init_message_broker_components_with_publisher(&config.redis_url).await;

        let events_queue = Arc::new(events_queue);
        let quests_channel = Arc::new(quests_channel);

        let database = create_quests_db_component(&config.database_url)
            .await
            .map_err(|_| "Couldn't connect to the database".to_string())?;
        let database = Arc::new(database);

        Ok(Self {
            events_queue,
            quests_channel,
            database,
        })
    }

    pub async fn process(&self) -> Result<JoinHandle<ProcessEventResult>, Error> {
        let event = self.events_queue.pop().await?;
        Ok(tokio::spawn(process_event(
            event,
            self.quests_channel.clone(),
            self.database.clone(),
            self.events_queue.clone(),
        )))
    }
}

/// Starts the main processing task which reads events from the queue, updates the quest states and
/// publishes the changes.
///
/// Panics if can't parse the config
pub async fn start_event_processing() -> EventProcessingResult<()> {
    initialize_logger();

    // TODO: read from config
    let config = Config::new().expect("Can parse config");
    let event_processor = EventProcessor::from_config(&config).await?;

    info!("Listening for events to process...");
    loop {
        if let Err(err) = event_processor.process().await {
            error!("Couldn't spawn task to process event due error: {err:?}");
        }
    }
}
