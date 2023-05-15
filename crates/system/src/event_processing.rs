use std::sync::Arc;

use log::{debug, error, info};
use quests_db::{
    core::{
        definitions::{AddEvent, QuestInstance, QuestsDatabase},
        errors::DBError,
    },
    create_quests_db_component, Database,
};
use quests_message_broker::{
    channel::{ChannelPublisher, RedisChannelPublisher},
    init_message_broker_components_with_publisher,
    messages_queue::{MessagesQueue, RedisMessagesQueue},
};
use quests_protocol::{definitions::*, quests::*};
use tokio::task::JoinHandle;

use crate::configuration::Config;

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

        let database = create_quests_db_component(&config.database_url, false)
            .await
            .map_err(|_| "Couldn't connect to the database".to_string())?;

        let events_queue = Arc::new(events_queue);
        let quests_channel = Arc::new(quests_channel);
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
        let quest_instances = self
            .database
            .get_active_user_quest_instances(&event.address)
            .await
            .map_err(|err| {
                debug!(
                "Processing event > Couldn't retrieve quests for user with address {:?}: {err:?}",
                event.address
            );
                ProcessEventError::Failed
            })?;

        info!(
            "Processing event > About to test event against {} instances",
            quest_instances.len()
        );

        let mut event_applied_to_instances = 0;
        for quest_instance in quest_instances {
            debug!("Processing event > for instance {quest_instance:?}");
            let quest = self.get_quest(&quest_instance.quest_id).await?;

            debug!("Processing event > for instance with quest {quest:?}");
            match self
                .process_event_for_quest_instance(&quest, &quest_instance, event)
                .await
            {
                Ok(ApplyEventResult::NewState(quest_state)) => {
                    match self
                        .add_event_and_notify(event, &quest_instance, quest, quest_state)
                        .await
                    {
                        Ok(_) => event_applied_to_instances += 1,
                        Err(err) => {
                            error!(
                                "Processing event > Couldn't add event to instance {}: {err:?}",
                                &quest_instance.id
                            );
                        }
                    };
                }
                Ok(ApplyEventResult::Ignored) => info!(
                    "Processing event > Event for quest instance {} was ignored",
                    &quest_instance.id
                ),
                Err(err) => {
                    info!(
                    "Processing event > Failed to process event for quest instance id: {} with err: {err:?}",
                    quest_instance.id,
                );
                }
            }
        }

        Ok(event_applied_to_instances)
    }

    async fn add_event_and_notify(
        self: &Arc<Self>,
        event: &Event,
        quest_instance: &QuestInstance,
        quest: Quest,
        quest_state: QuestState,
    ) -> Result<(), ProcessEventError> {
        debug!("Processing event > event applied with new state: {quest_state:?}");
        let add_event = AddEvent {
            id: event.id.clone(),
            user_address: &event.address,
            event: event.encode_to_vec(),
        };

        debug!(
            "Processing event > adding event for instance: {:?}",
            quest_instance.id
        );
        self.database
            .add_event(&add_event, &quest_instance.id)
            .await?;

        debug!(
            "Processing event > publishing user update for instance: {:?}",
            quest_instance.id
        );
        self.quests_channel
            .publish(UserUpdate {
                message: Some(user_update::Message::QuestStateUpdate(QuestStateUpdate {
                    quest_data: Some(QuestStateWithData {
                        name: quest.name,
                        description: quest.description,
                        quest_instance_id: quest_instance.id.clone(),
                        quest_state: Some(quest_state),
                    }),
                    event_id: event.id.clone(),
                })),
            })
            .await;

        Ok(())
    }

    async fn get_quest(self: &Arc<Self>, quest_id: &str) -> Result<Quest, ProcessEventError> {
        debug!("Processing event > Getting quest with id: {quest_id:?}");
        let quest = self.database.get_quest(quest_id).await?;

        let quest_definition = QuestDefinition::decode(&*quest.definition)?;
        let quest = Quest {
            name: quest.name,
            description: quest.description,
            definition: Some(quest_definition),
        };
        Ok(quest)
    }

    // TODO: handle concurrent events with different timestamps
    async fn process_event_for_quest_instance(
        self: &Arc<Self>,
        quest: &Quest,
        quest_instance: &QuestInstance,
        event: &Event,
    ) -> Result<ApplyEventResult, ProcessEventError> {
        debug!("Processing event > Quest instance > Retrieving old events");
        let last_events = self.database.get_events(&quest_instance.id).await?;
        let mut events = vec![];
        for past_event in last_events {
            events.push(Event::decode(&*past_event.event)?);
        }

        debug!("Processing event > Quest instance > About to apply old events");
        let quest_graph = QuestGraph::from(quest);
        let current_state = get_state(quest, events);

        debug!("Processing event > Quest instance > About to apply new event");
        let new_state = current_state.apply_event(&quest_graph, event);

        Ok(if current_state == new_state {
            ApplyEventResult::Ignored
        } else {
            ApplyEventResult::NewState(new_state)
        })
    }
}

/// Starts the main processing task which reads events from the queue, updates the quest states and
/// publishes the changes.
///
/// Panics if can't parse the config
pub async fn start_event_processing(config: &Config) -> JoinHandle<EventProcessingResult<()>> {
    let config = config.clone();
    tokio::spawn(async move {
        let event_processor = EventProcessor::from_config(&config).await?;
        let event_processor = Arc::new(event_processor);

        info!("Listening for events to process...");
        loop {
            if let Err(err) = event_processor.clone().process().await {
                error!("Couldn't spawn task to process event due error: {err:?}");
            }
        }
    })
}
pub enum ApplyEventResult {
    NewState(QuestState),
    Ignored,
}

#[derive(Debug)]
pub enum ProcessEventError {
    Serialization,
    DatabaseAccess,
    Failed,
}

pub type ProcessEventResult = Result<usize, ProcessEventError>;

impl From<ProtocolDecodeError> for ProcessEventError {
    fn from(_value: ProtocolDecodeError) -> Self {
        ProcessEventError::Serialization
    }
}

impl From<DBError> for ProcessEventError {
    fn from(_value: DBError) -> Self {
        ProcessEventError::DatabaseAccess
    }
}
