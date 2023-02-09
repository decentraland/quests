use std::sync::Arc;

use log::info;
use quests_db::core::definitions::{AddEvent, QuestInstance, QuestsDatabase};
use quests_definitions::quests::Event;

use quests_message_broker::{
    events_queue::EventsQueue,
    quests_channel::{QuestUpdate, QuestsChannel, QuestState},
};

pub enum ProcessEventResult {
    NewState(QuestState),
    Ignored,
}

pub async fn process_event(
    event: Event,
    quests_channel: Arc<impl QuestsChannel>,
    database: Arc<impl QuestsDatabase>,
    events_queue: Arc<impl EventsQueue>,
) {
    // get user quest instances
    let quest_instances = database.get_user_quest_instances(&event.address).await;

    match quest_instances {
        Ok(quest_instances) => {
            for quest_instance in quest_instances {
                match process_event_for_quest_instance(&quest_instance, &event, database.clone())
                    .await
                {
                    ProcessEventResult::NewState(quest_state) => {
                        let add_event = AddEvent {
                            user_address: &event.address,
                            event: bincode::serialize(&event).expect("can serialize event"), // TODO: error handling
                        };
                        database.add_event(&add_event, &quest_instance.id).await;
                        quests_channel
                            .publish(&quest_instance.id, QuestUpdate { state: quest_state })
                            .await;
                    }
                    ProcessEventResult::Ignored => info!(
                        "Event {:?} for quest instance {} was ignored",
                        event, &quest_instance.id
                    ),
                }
            }
        }
        Err(_) => {
            info!(
                "Couldn't retrieve quests for user with address {:?}",
                event.address
            );

            // TODO: should we retry here?
            events_queue.push(event).await;
        }
    }
}

// TODO: handle concurrent events with different timestamps
async fn process_event_for_quest_instance(
    quest_instance: &QuestInstance,
    event: &Event,
    database: Arc<impl QuestsDatabase>,
) -> ProcessEventResult {
    // try to apply event to every instance
    let events = database.get_events(&quest_instance.id).await;
    let quest = database.get_quest(&quest_instance.quest_id).await.expect("Can retrieve quest"); // TODO: error handling
    let quest_definition = bincode::deserialize::<Quest>(&quest.definition);
    let initial_state = quest_definition.get_initial_state();
    for event in events {
        // apply event and get new state
        // definitions::apply_event(state, event) -> state
    }
    // get all quest instance events
    // apply all of them to get current state
    todo!()
}
