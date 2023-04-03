use log::info;
use quests_db::core::{
    definitions::{AddEvent, QuestInstance, QuestsDatabase},
    errors::DBError,
};
use quests_message_broker::{channel::ChannelPublisher, messages_queue::MessagesQueue};
use quests_protocol::{
    quest_graph::QuestGraph,
    quest_state::get_state,
    quests::{
        user_update, Event, Quest, QuestDefinition, QuestState, QuestStateUpdate, UserUpdate,
    },
    ProtocolDecodeError, ProtocolMessage,
};
use std::sync::Arc;
use tokio::sync::Mutex;

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

pub async fn process_event(
    event: Event,
    quests_channel: Arc<Mutex<impl ChannelPublisher<UserUpdate> + ?Sized>>,
    database: Arc<impl QuestsDatabase + ?Sized>,
    events_queue: Arc<impl MessagesQueue<Event> + ?Sized>,
) -> ProcessEventResult {
    // get user quest instances
    let quest_instances = database.get_user_quest_instances(&event.address).await;

    match quest_instances {
        Ok(quest_instances) => {
            let mut event_applied_to_instances = 0;
            for quest_instance in quest_instances {
                match process_event_for_quest_instance(&quest_instance, &event, database.clone())
                    .await
                {
                    Ok(ApplyEventResult::NewState(quest_state)) => {
                        let add_event = AddEvent {
                            user_address: &event.address,
                            event: event.encode_to_vec(),
                        };
                        database.add_event(&add_event, &quest_instance.id).await?;
                        quests_channel
                            .lock()
                            .await
                            .publish(UserUpdate {
                                message: Some(user_update::Message::QuestState(QuestStateUpdate {
                                    quest_instance_id: quest_instance.id.clone(),
                                    quest_state: Some(quest_state),
                                })),
                            })
                            .await;
                        event_applied_to_instances += 1;
                    }
                    Ok(ApplyEventResult::Ignored) => info!(
                        "Event for quest instance {} was ignored",
                        &quest_instance.id
                    ),
                    Err(e) => {
                        info!(
                            "Failed to process event for quest instance id: {} with err: {:?}",
                            quest_instance.id, e
                        );
                    }
                }
            }
            Ok(event_applied_to_instances)
        }
        Err(_) => {
            info!(
                "Couldn't retrieve quests for user with address {:?}",
                event.address
            );

            // TODO: should we retry here?
            let _ = events_queue.push(&event).await;
            Err(ProcessEventError::Failed)
        }
    }
}

// TODO: handle concurrent events with different timestamps
async fn process_event_for_quest_instance(
    quest_instance: &QuestInstance,
    event: &Event,
    database: Arc<impl QuestsDatabase + ?Sized>,
) -> Result<ApplyEventResult, ProcessEventError> {
    // try to apply event to every instance
    let quest = database.get_quest(&quest_instance.quest_id).await?;
    let quest_definition = QuestDefinition::decode(&*quest.definition)?;
    let quest = Quest {
        name: quest.name,
        description: quest.description,
        definition: quest_definition,
    };

    let last_events = database.get_events(&quest_instance.id).await?;
    let mut events = vec![];
    for past_event in last_events {
        events.push(Event::decode(&*past_event.event)?);
    }

    let quest_graph = QuestGraph::from(&quest);
    let current_state = get_state(&quest, events);

    let new_state = current_state.apply_event(&quest_graph, event);

    Ok(if current_state == new_state {
        ApplyEventResult::Ignored
    } else {
        ApplyEventResult::NewState(new_state)
    })
}
