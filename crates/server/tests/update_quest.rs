mod common;
use actix_web::test::{call_service, init_service, read_body_json, TestRequest};
pub use common::*;
use quests_db::core::definitions::{CreateQuest, QuestsDatabase};
use quests_db::create_quests_db_component;
use quests_definitions::quests::{Action, Coordinates, Quest, QuestDefinition, Step, Tasks};
use quests_server::api::routes::ErrorResponse;

#[actix_web::test]
async fn update_quest_should_be_200() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url)
        .await
        .unwrap();

    let app = init_service(build_app(&config).await).await;
    let quest = quest_samples::grab_some_apples();

    let create_quest = CreateQuest {
        name: &quest.name,
        description: &quest.description,
        definition: bincode::serialize(&quest.definition).unwrap(),
    };

    let id = db.create_quest(&create_quest).await.unwrap();

    let quest_update = Quest {
        name: "QUEST-1_UPDATE".to_string(),
        description: "Grab some apples - Updated".to_string(),
        definition: QuestDefinition {
            connections: vec![
                ("A-Updated".to_string(), "B".to_string()),
                ("B".to_string(), "C".to_string()),
                ("C".to_string(), "D".to_string()),
            ],
            steps: vec![
                Step {
                    id: "A-Updated".to_string(),
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
                            coordinates: Coordinates(10, 20),
                        }],
                    },
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: Tasks::Single {
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(10, 20),
                        }],
                    },
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: Tasks::Single {
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(10, 20),
                        }],
                    },
                    on_complete_hook: None,
                    description: "".to_string(),
                },
            ],
        },
    };

    let req = TestRequest::put()
        .uri(format!("/quests/{}", id).as_str())
        .set_json(&quest_update)
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success());

    let quest_updated = db.get_quest(&id).await.unwrap();

    assert_eq!(quest_updated.name, "QUEST-1_UPDATE");
    assert_eq!(quest_updated.description, "Grab some apples - Updated");
    let definition: QuestDefinition = bincode::deserialize(&quest_updated.definition).unwrap();
    assert_eq!(quest_update.definition.steps.len(), definition.steps.len());
    for step in &quest_update.definition.steps {
        assert!(definition.steps.iter().any(|s| s.id == step.id));
    }
    assert_eq!(quest_update.definition.connections, definition.connections);
}

#[actix_web::test]
async fn update_quest_should_be_400_uuid_bad_format() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;
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
                            coordinates: Coordinates(10, 20),
                        }],
                    },
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: Tasks::Single {
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(10, 20),
                        }],
                    },
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: Tasks::Single {
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(10, 20),
                        }],
                    },
                    on_complete_hook: None,
                    description: "".to_string(),
                },
            ],
        },
    };

    let quest_update = Quest {
        name: "QUEST-1_UPDATE".to_string(),
        ..quest_definition
    };

    let req = TestRequest::put()
        .uri("/quests/1aa")
        .set_json(quest_update)
        .to_request();

    let response = call_service(&app, req).await;

    assert_eq!(response.status(), 400);
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 400);
    assert!(body.message.contains("Bad Request:"));
}

#[actix_web::test]
async fn update_quest_should_be_400_quest_validation_error() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;
    let quest_definition = Quest {
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
        definition: QuestDefinition {
            connections: vec![], // not needed for test
            steps: vec![],       // not needed for this test
        },
    };

    let quest_update = Quest {
        name: "QUEST-1_UPDATE".to_string(),
        ..quest_definition
    };

    let req = TestRequest::put()
        .uri("/quests/whatever-uuid-because-it-fails-due-to-validation")
        .set_json(quest_update)
        .to_request();

    let response = call_service(&app, req).await;

    assert_eq!(response.status(), 400);
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 400);
    assert!(body
        .message
        .contains("Quest Validation Error: Missing the definition for the quest"));
}
