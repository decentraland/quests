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
    quests_channel: Arc<impl ChannelPublisher<UserUpdate> + ?Sized>,
    database: Arc<impl QuestsDatabase + ?Sized>,
    events_queue: Arc<impl MessagesQueue<Event> + ?Sized>,
) -> ProcessEventResult {
    info!("Processing event > {event:?}");

    // get user quest instances
    let quest_instances = database.get_user_quest_instances(&event.address).await;

    match quest_instances {
        Ok(quest_instances) => {
            let mut event_applied_to_instances = 0;
            for quest_instance in quest_instances {
                info!("Processing event > for instance {quest_instance:?}");
                let quest = get_quest(&quest_instance.quest_id, database.clone()).await?;

                info!("Processing event > for instance with quest {quest:?}");
                match process_event_for_quest_instance(
                    &quest,
                    &quest_instance,
                    &event,
                    database.clone(),
                )
                .await
                {
                    Ok(ApplyEventResult::NewState(quest_state)) => {
                        info!("Processing event > event applied with new state: {quest_state:?}");
                        let add_event = AddEvent {
                            user_address: &event.address,
                            event: event.encode_to_vec(),
                        };
                        database.add_event(&add_event, &quest_instance.id).await?;

                        quests_channel
                            .publish(UserUpdate {
                                message: Some(user_update::Message::QuestState(QuestStateUpdate {
                                    name: quest.name,
                                    description: quest.description,
                                    quest_instance_id: quest_instance.id.clone(),
                                    quest_state: Some(quest_state),
                                })),
                            })
                            .await;
                        info!(
                            "Processing event > publishing user update for instance: {:?}",
                            quest_instance.id
                        );

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

async fn get_quest(
    quest_id: &str,
    database: Arc<impl QuestsDatabase + ?Sized>,
) -> Result<Quest, ProcessEventError> {
    info!("Processing event > Getting quest with id: {quest_id:?}");
    let quest = database.get_quest(quest_id).await?;

    let quest_definition = QuestDefinition::decode(&*quest.definition)?;
    let quest = Quest {
        name: quest.name,
        description: quest.description,
        definition: quest_definition,
    };
    Ok(quest)
}

// TODO: handle concurrent events with different timestamps
async fn process_event_for_quest_instance(
    quest: &Quest,
    quest_instance: &QuestInstance,
    event: &Event,
    database: Arc<impl QuestsDatabase + ?Sized>,
) -> Result<ApplyEventResult, ProcessEventError> {
    // try to apply event to every instance
    info!("Processing event > Quest instance > Retrieving old events");
    let last_events = database.get_events(&quest_instance.id).await?;
    let mut events = vec![];
    for past_event in last_events {
        events.push(Event::decode(&*past_event.event)?);
    }

    info!("Processing event > Quest instance > About to apply old event");
    let quest_graph = QuestGraph::from(quest);
    let current_state = get_state(quest, events);

    info!("Processing event > Quest instance > About to apply new event");
    let new_state = current_state.apply_event(&quest_graph, event);

    Ok(if current_state == new_state {
        ApplyEventResult::Ignored
    } else {
        ApplyEventResult::NewState(new_state)
    })
}
