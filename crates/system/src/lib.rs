pub mod configuration;
mod event_processing;

use configuration::Config;
use env_logger::init as initialize_logger;
use event_processing::{process_event, ProcessEventResult};
use log::{error, info};
use quests_db::core::definitions::QuestsDatabase;
use quests_db::create_quests_db_component;
use quests_message_broker::channel::ChannelPublisher;
use quests_message_broker::init_message_broker_components_with_publisher;
use quests_message_broker::messages_queue::MessagesQueue;
use quests_protocol::quests::{Event, UserUpdate};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

pub type Error = String;
pub type EventProcessingResult<T> = Result<T, Error>;

pub struct EventProcessor {
    pub events_queue: Arc<dyn MessagesQueue<Event>>,
    quests_channel: Arc<Mutex<dyn ChannelPublisher<UserUpdate>>>,
    database: Arc<dyn QuestsDatabase>,
}

impl EventProcessor {
    pub async fn from_config(config: &Config) -> EventProcessingResult<Self> {
        let (events_queue, quests_channel) =
            init_message_broker_components_with_publisher(&config.redis_url).await;

        let events_queue = Arc::new(events_queue);
        let quests_channel = Arc::new(Mutex::new(quests_channel));

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

    info!("Listening to events to process...");
    loop {
        let _ = process(&event_processor).await;
    }
}

pub async fn process(
    event_processor: &EventProcessor,
) -> Result<JoinHandle<ProcessEventResult>, Error> {
    // Read items from events queue
    let event = event_processor.events_queue.pop().await;
    match event {
        Ok(event) => {
            // Spawn task to process the event
            Ok(tokio::spawn(process_event(
                event,
                event_processor.quests_channel.clone(),
                event_processor.database.clone(),
                event_processor.events_queue.clone(),
            )))
        }
        Err(reason) => {
            error!("Pop event error: {}", reason);
            Err(reason)
        }
    }
}
