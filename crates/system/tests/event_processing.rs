use quests_db::{core::definitions::*, create_quests_db_component};
use quests_definitions::quests::{Event, *};
use quests_system::{configuration::Config, EventProcessor};

use crate::common::database::create_test_db;

mod common;

#[tokio::test]
async fn can_process_events() {
    env_logger::init();
    let db_url = create_test_db().await;
    let db = create_quests_db_component(&db_url)
        .await
        .expect("can create db");

    let quest_definition = Quest {
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
        definition: QuestDefinition {
            connections: vec![
                ("A".to_string(), "B".to_string()),
                ("B".to_string(), "C".to_string()),
                ("C".to_string(), "D".to_string()),
            ],
            steps: vec![
                Step {
                    id: "A".to_string(),
                    tasks: Tasks::Single {
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(10, 20),
                        }],
                    },
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: Tasks::Single {
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(13, 20),
                        }],
                    },
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: Tasks::Single {
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(10, 24),
                        }],
                    },
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: Tasks::Single {
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(40, 20),
                        }],
                    },
                    on_complete_hook: None,
                    description: "".to_string(),
                },
            ],
        },
    };

    let create_quest = CreateQuest {
        name: &quest_definition.name,
        description: &quest_definition.description,
        definition: bincode::serialize(&quest_definition.definition).unwrap(),
    };

    let result = db.create_quest(&create_quest).await;
    assert!(result.is_ok());

    let user_address = "0xA";
    let result = db.start_quest(&result.unwrap(), user_address).await;
    assert!(result.is_ok());

    let quest_instance_id = result.unwrap();

    println!("quest instance id: {quest_instance_id}");

    let mut config = Config::new().expect("Can parse config");
    config.database_url = db_url;
    let event_processor = EventProcessor::from_config(&config)
        .await
        .expect("can initialize event processor");

    let action = Action::Location {
        coordinates: Coordinates(10, 20),
    };

    let event = Event {
        address: user_address.to_string(),
        timestamp: 0,
        action,
    };

    println!("about to push event");
    let _ = event_processor.events_queue.push(&event).await;

    let process_event = quests_system::process(&event_processor)
        .await
        .expect("can process event");
    let result = process_event.await;

    match result {
        Ok(Ok(result)) => {
            assert_eq!(result, 1);
        }
        _ => panic!("Couldn't process event"),
    }

    let events = db
        .get_events(&quest_instance_id)
        .await
        .expect("can retrieve events");
    assert!(events.len() == 1);
}
