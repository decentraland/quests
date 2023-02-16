use std::sync::Arc;

use log::info;
use quests_db::core::definitions::{AddEvent, QuestInstance, QuestsDatabase};
use quests_definitions::{
    quest_graph::QuestGraph,
    quest_state::{get_state, QuestState},
    quests::{Event, Quest, QuestDefinition},
};

use quests_message_broker::{
    events_queue::EventsQueue,
    quests_channel::{QuestUpdate, QuestsChannel},
};
use tokio::sync::Mutex;

pub enum ProcessEventResult {
    NewState(QuestState),
    Ignored,
}

pub async fn process_event(
    event: Event,
    quests_channel: Arc<Mutex<impl QuestsChannel + ?Sized>>,
    database: Arc<impl QuestsDatabase + ?Sized>,
    events_queue: Arc<impl EventsQueue + ?Sized>,
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
                        database
                            .add_event(&add_event, &quest_instance.id)
                            .await
                            .unwrap(); // TODO: Error handling
                        quests_channel
                            .lock()
                            .await
                            .publish(&quest_instance.id, QuestUpdate { state: quest_state })
                            .await;
                    }
                    ProcessEventResult::Ignored => info!(
                        "Event for quest instance {} was ignored",
                        &quest_instance.id
                    ),
                }
            }
        }
        Err(_) => {
            println!("FAIL");
            info!(
                "Couldn't retrieve quests for user with address {:?}",
                event.address
            );

            // TODO: should we retry here?
            let _ = events_queue.push(&event).await;
        }
    }
}

// TODO: handle concurrent events with different timestamps
async fn process_event_for_quest_instance(
    quest_instance: &QuestInstance,
    event: &Event,
    database: Arc<impl QuestsDatabase + ?Sized>,
) -> ProcessEventResult {
    // try to apply event to every instance
    let events = database.get_events(&quest_instance.id).await.unwrap(); // TODO: error handling
    let quest = database
        .get_quest(&quest_instance.quest_id)
        .await
        .expect("Can retrieve quest"); // TODO: error handling
    let quest_definition = bincode::deserialize::<QuestDefinition>(&quest.definition).unwrap(); // TODO: error handling
    let quest = Quest {
        name: quest.name,
        description: quest.description,
        definition: quest_definition,
    };

    let events = events
        .iter()
        .map(|event| bincode::deserialize::<Event>(&event.event).unwrap())
        .collect();

    let quest_graph = QuestGraph::from_quest(&quest);
    let current_state = get_state(&quest, events);
    let new_state = current_state.apply_event(&quest_graph, event);

    if current_state == new_state {
        ProcessEventResult::Ignored
    } else {
        ProcessEventResult::NewState(new_state)
    }
}
