use std::sync::Arc;

use quests_db::{core::definitions::*, create_quests_db_component};
use quests_message_broker::messages_queue::MessagesQueue;
use quests_protocol::{
    definitions::{Event as ProtoEvent, *},
    quests::Coordinates,
};
use quests_system::{configuration::Config, event_processing::EventProcessor};

use crate::common::database::create_test_db;

mod common;

#[tokio::test]
async fn can_process_events() {
    env_logger::init();
    let db_url = create_test_db().await;
    let db = create_quests_db_component(&db_url, true)
        .await
        .expect("can create db");

    let quest_definition = Quest {
        id: "1e9a8bbf-2223-4f51-b7e5-660d35cedef4".to_string(),
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
        creator_address: "0xB".to_string(),
        definition: Some(QuestDefinition {
            connections: vec![
                Connection::new("A", "B"),
                Connection::new("B", "C"),
                Connection::new("C", "D"),
            ],
            steps: vec![
                Step {
                    id: "A".to_string(),
                    tasks: vec![Task {
                        id: "A_1".to_string(),
                        description: "".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: vec![Task {
                        id: "B_1".to_string(),
                        description: "".to_string(),
                        action_items: vec![Action::location(Coordinates::new(13, 20))],
                    }],
                    description: "".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: vec![Task {
                        id: "C_1".to_string(),
                        description: "".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 24))],
                    }],
                    description: "".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: vec![Task {
                        id: "D_1".to_string(),
                        description: "".to_string(),
                        action_items: vec![Action::location(Coordinates::new(40, 20))],
                    }],
                    description: "".to_string(),
                },
            ],
        }),
    };

    let create_quest = CreateQuest {
        name: &quest_definition.name,
        description: &quest_definition.description,
        definition: quest_definition
            .definition
            .as_ref()
            .unwrap()
            .encode_to_vec(),
    };

    let result = db.create_quest(&create_quest, "0xA").await;
    assert!(result.is_ok());

    let user_address = "0xB";
    let result = db.start_quest(&result.unwrap(), user_address).await;
    assert!(result.is_ok());

    let quest_instance_id = result.unwrap();

    let mut config = Config::new().expect("Can parse config");
    config.database_url = db_url;
    let event_processor = EventProcessor::from_config(&config)
        .await
        .expect("can initialize event processor");

    let action = Action::location(Coordinates::new(10, 20));

    let event = ProtoEvent {
        id: uuid::Uuid::new_v4().to_string(),
        address: user_address.to_string(),
        action: Some(action),
    };

    event_processor
        .events_queue
        .push(&event)
        .await
        .expect("can push event");

    let event_processor = Arc::new(event_processor);

    let result = event_processor
        .process()
        .await
        .expect("can spawn task to process event")
        .await
        .expect("can await join handle")
        .expect("can process event");

    assert_eq!(result, 1);

    let events = db
        .get_events(&quest_instance_id)
        .await
        .expect("can retrieve events");
    assert!(events.len() == 1);
}
