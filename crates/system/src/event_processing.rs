use crate::rewards::give_rewards_to_user;
use log::{debug, error, info};
use quests_db::{
    core::{
        definitions::{AddEvent, QuestsDatabase},
        errors::DBError,
    },
    create_quests_db_component, Database,
};
use quests_message_broker::{
    channel::{ChannelPublisher, RedisChannelPublisher},
    messages_queue::{MessagesQueue, RedisMessagesQueue},
    redis::Redis,
};
use quests_protocol::{definitions::*, quests::*};
use std::sync::Arc;
use tokio::task::JoinHandle;

use crate::{
    configuration::Config,
    quests::{get_all_quest_states_by_user_address, QuestStateCalculationError},
    QUESTS_CHANNEL_NAME, QUESTS_EVENTS_QUEUE_NAME,
};

pub type Error = String;
pub type EventProcessingResult<T> = Result<T, Error>;

pub struct EventProcessor {
    pub events_queue: Arc<RedisMessagesQueue>,
    quests_channel: Arc<RedisChannelPublisher>,
    database: Arc<Database>,
}

impl EventProcessor {
    pub fn from(
        events_queue: Arc<RedisMessagesQueue>,
        quests_channel: Arc<RedisChannelPublisher>,
        database: Arc<Database>,
    ) -> Self {
        Self {
            events_queue,
            quests_channel,
            database,
        }
    }

    pub async fn from_config(config: &Config) -> EventProcessingResult<Self> {
        let redis = Redis::new(&config.redis_url)
            .await
            .map_err(|_| "Couldn't initialize redis connection".to_string())?;
        let redis = Arc::new(redis);

        let events_queue = RedisMessagesQueue::new(redis.clone(), QUESTS_EVENTS_QUEUE_NAME);
        let events_queue = Arc::new(events_queue);

        let quests_channel = RedisChannelPublisher::new(redis.clone(), QUESTS_CHANNEL_NAME);
        let quests_channel = Arc::new(quests_channel);

        let database = create_quests_db_component(&config.database_url, false)
            .await
            .map_err(|_| "Couldn't connect to the database".to_string())?;
        let database = Arc::new(database);

        Ok(Self {
            events_queue,
            quests_channel,
            database,
        })
    }

    pub async fn process(self: Arc<Self>) -> Result<JoinHandle<ProcessEventResult>, Error> {
        let event = self.events_queue.pop().await?;
        Ok(tokio::spawn(self.process_event(event)))
    }

    pub async fn process_event(self: Arc<Self>, event: Event) -> ProcessEventResult {
        info!("Processing event > {event:?}");
        let result = self.do_process_event(&event).await;
        match &result {
            Ok(instances_applied) => {
                info!("Processing event > Event applied to {instances_applied} instances")
            }
            Err(err) => {
                let _ = self.events_queue.push(&event).await;
                error!("Processing event > Couldn't process event {:?}", err);
            }
        }
        result
    }

    async fn do_process_event(self: &Arc<Self>, event: &Event) -> Result<usize, ProcessEventError> {
        let quest_instances =
            get_all_quest_states_by_user_address(self.database.clone(), &event.address).await?;

        info!(
            "Processing event > About to test event against {} instances",
            quest_instances.len()
        );

        let mut event_applied_to_instances = 0;
        for (instance_id, (quest, quest_state)) in quest_instances {
            debug!("Processing event > for instance {:?}", instance_id);

            if quest_state.is_completed() {
                continue;
            }

            debug!("Processing event > quest state: \n {quest_state:#?} \n event: {event:#?}");

            let quest_graph = QuestGraph::from(&quest);
            let new_state = quest_state.apply_event(&quest_graph, event);
            if new_state != quest_state {
                match self
                    .add_event_and_notify(event, &quest.id, &instance_id, new_state)
                    .await
                {
                    Ok(_) => event_applied_to_instances += 1,
                    Err(err) => {
                        error!(
                            "Processing event > Couldn't add event to instance {}: {err:?}",
                            &instance_id
                        );
                    }
                };
            }
        }

        if event_applied_to_instances == 0 {
            self.quests_channel
                .publish(UserUpdate {
                    user_address: event.address.clone(),
                    message: Some(user_update::Message::EventIgnored(event.id.clone())),
                })
                .await;
            info!("Processing event > Event was ignored");
        }

        Ok(event_applied_to_instances)
    }

    async fn add_event_and_notify(
        self: &Arc<Self>,
        event: &Event,
        quest_id: &str,
        quest_instance_id: &str,
        mut quest_state: QuestState,
    ) -> Result<(), ProcessEventError> {
        debug!("Processing event > event applied with new state: {quest_state:?}");
        let add_event = AddEvent {
            id: event.id.clone(),
            user_address: &event.address,
            event: event.encode_to_vec(),
        };

        debug!(
            "Processing event > adding event for instance: {:?}",
            quest_instance_id
        );
        self.database
            .add_event(&add_event, quest_instance_id)
            .await?;

        if quest_state.is_completed() {
            debug!("Processing event > Calling rewards hook");
            give_rewards_to_user(self.database.clone(), quest_id, &event.address).await;
            debug!("Processing event > recording instance as completed");
            if let Err(err) = self
                .database
                .complete_quest_instance(quest_instance_id)
                .await
            {
                error!(
                    "Processing event > Failed to record instance: {quest_instance_id} as completed: {err}",
                );
            }
        }

        quest_state.hide_actions();
        self.quests_channel
            .publish(UserUpdate {
                message: Some(user_update::Message::QuestStateUpdate(QuestStateUpdate {
                    instance_id: quest_instance_id.to_string(),
                    quest_state: Some(quest_state),
                    event_id: event.id.clone(),
                })),
                user_address: event.address.clone(),
            })
            .await;

        Ok(())
    }
}

pub fn run_event_processor(
    database: Arc<Database>,
    events_queue: Arc<RedisMessagesQueue>,
    quests_channel_publisher: Arc<RedisChannelPublisher>,
) -> JoinHandle<EventProcessingResult<()>> {
    let event_processor = EventProcessor::from(events_queue, quests_channel_publisher, database);

    start_event_processing(event_processor)
}

/// Starts the main processing task which reads events from the queue, updates the quest states and
/// publishes the changes.
pub(crate) fn start_event_processing(
    event_processor: EventProcessor,
) -> JoinHandle<EventProcessingResult<()>> {
    let event_processor = Arc::new(event_processor);
    tokio::spawn(async move {
        info!("Listening for events to process...");
        loop {
            if let Err(err) = event_processor.clone().process().await {
                error!("Couldn't spawn task to process event due error: {err:?}");
            }
        }
    })
}

#[derive(Debug)]
pub enum ProcessEventError {
    Serialization,
    DatabaseAccess(DBError),
    Failed,
}

pub type ProcessEventResult = Result<usize, ProcessEventError>;

impl From<ProtocolDecodeError> for ProcessEventError {
    fn from(_value: ProtocolDecodeError) -> Self {
        ProcessEventError::Serialization
    }
}

impl From<DBError> for ProcessEventError {
    fn from(value: DBError) -> Self {
        ProcessEventError::DatabaseAccess(value)
    }
}

impl From<QuestStateCalculationError> for ProcessEventError {
    fn from(value: QuestStateCalculationError) -> Self {
        match value {
            QuestStateCalculationError::DatabaseError(e) => ProcessEventError::DatabaseAccess(e),
            QuestStateCalculationError::DefinitionError => ProcessEventError::Serialization,
            QuestStateCalculationError::StateError => ProcessEventError::Failed,
        }
    }
}
