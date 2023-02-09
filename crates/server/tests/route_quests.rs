mod common;
use actix_web::{
    test::{call_service, init_service, read_body_json, TestRequest},
    web::Data,
};
pub use common::*;
use quests_db::core::definitions::{CreateQuest, QuestsDatabase};
use quests_db::create_quests_db_component;
use quests_definitions::quests::{Quest, QuestDefinition, Step, Tasks};
use quests_server::{get_app_router, routes::ErrorResponse};

#[actix_web::test]
async fn create_quest_should_be_200() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url)
        .await
        .unwrap();

    let app = init_service(get_app_router(&Data::new(config), &Data::new(db))).await;

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
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
            ],
        },
    };

    let req = TestRequest::post()
        .uri("/quests")
        .set_json(quest_definition)
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success())
}

#[actix_web::test]
async fn create_quest_should_be_400_quest_validation_error() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url)
        .await
        .unwrap();

    let app = init_service(get_app_router(&Data::new(config), &Data::new(db))).await;

    let quest_definition = Quest {
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
        definition: QuestDefinition {
            connections: vec![],
            steps: vec![],
        },
    };

    let req = TestRequest::post()
        .uri("/quests")
        .set_json(quest_definition)
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_client_error());
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 400);
    assert!(body
        .message
        .contains("Quest Validation Error: Missing the definition for the quest"));
}

#[actix_web::test]
async fn get_quests_should_be_200() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url)
        .await
        .unwrap();

    let app = init_service(get_app_router(&Data::new(config), &Data::new(db))).await;

    let quest_definition = Quest {
        name: "QUEST-2".to_string(),
        description: "Grab some pies".to_string(),
        definition: QuestDefinition {
            connections: vec![
                ("A".to_string(), "B".to_string()),
                ("B".to_string(), "C".to_string()),
                ("C".to_string(), "D".to_string()),
            ],
            steps: vec![
                Step {
                    id: "A".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
            ],
        },
    };

    let req = TestRequest::post()
        .uri("/quests")
        .set_json(quest_definition)
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success());

    let req = TestRequest::get()
        .uri("/quests?offset=0&limit=2")
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success());
}

#[actix_web::test]
async fn get_quests_should_be_400() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url)
        .await
        .unwrap();

    let app = init_service(get_app_router(&Data::new(config), &Data::new(db))).await;

    let req = TestRequest::get().uri("/quests?offset=0aa").to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_client_error());

    let req = TestRequest::get().uri("/quests?limit=0aa").to_request();

    let response = call_service(&app, req).await;

    assert_eq!(response.status(), 400);
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 400);
    assert!(body.message.contains("Bad Request:"));

    let req = TestRequest::get()
        .uri("/quests?offset=10a&limit=0aa")
        .to_request();

    let response = call_service(&app, req).await;

    assert_eq!(response.status(), 400);
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 400);
    assert!(body.message.contains("Bad Request:"));
}

#[actix_web::test]
async fn update_quest_should_be_200() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url)
        .await
        .unwrap();

    let quest = Quest {
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
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
            ],
        },
    };

    let create_quest = CreateQuest {
        name: &quest.name,
        description: &quest.description,
        definition: bincode::serialize(&quest.definition).unwrap(),
    };

    let id = db.create_quest(&create_quest).await.unwrap();

    let app = init_service(get_app_router(&Data::new(config), &Data::new(db.clone()))).await;

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
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: Tasks::None,
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
    let db = create_quests_db_component(&config.database_url)
        .await
        .unwrap();

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
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
            ],
        },
    };

    let app = init_service(get_app_router(&Data::new(config), &Data::new(db.clone()))).await;

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
    let db = create_quests_db_component(&config.database_url)
        .await
        .unwrap();

    let quest_definition = Quest {
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
        definition: QuestDefinition {
            connections: vec![], // not needed for test
            steps: vec![],       // not needed for this test
        },
    };

    let app = init_service(get_app_router(&Data::new(config), &Data::new(db.clone()))).await;

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

#[actix_web::test]
async fn delete_quest_should_be_200() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url)
        .await
        .unwrap();

    let quest_definition = Quest {
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
        definition: QuestDefinition {
            connections: vec![], // not needed for this test
            steps: vec![],       // not needed for this test
        },
    };

    let create_quest = CreateQuest {
        name: &quest_definition.name,
        description: &quest_definition.description,
        definition: vec![],
    };

    let id = db.create_quest(&create_quest).await.unwrap();

    let app = init_service(get_app_router(&Data::new(config), &Data::new(db.clone()))).await;

    let req = TestRequest::delete()
        .uri(format!("/quests/{}", id).as_str())
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success());

    assert!(db.get_quest(&id).await.is_err());
}

#[actix_web::test]
async fn delete_quest_should_be_400() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url)
        .await
        .unwrap();

    let app = init_service(get_app_router(&Data::new(config), &Data::new(db.clone()))).await;

    let req = TestRequest::delete().uri("/quests/1aab").to_request();

    let response = call_service(&app, req).await;

    assert_eq!(response.status(), 400);
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 400);
    assert!(body.message.contains("Bad Request:"));
}

#[actix_web::test]
async fn get_quest_should_be_200() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url)
        .await
        .unwrap();

    let quest_definition = Quest {
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
        definition: QuestDefinition {
            connections: vec![
                ("A".to_string(), "B".to_string()),
                ("B".to_string(), "C".to_string()),
            ],
            steps: vec![
                Step {
                    id: "A".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: Tasks::None,
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

    let id = db.create_quest(&create_quest).await.unwrap();

    let app = init_service(get_app_router(&Data::new(config), &Data::new(db.clone()))).await;

    let req = TestRequest::get()
        .uri(format!("/quests/{}", id).as_str())
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success());
    let body: Quest = read_body_json(response).await;
    assert_eq!(body.name, "QUEST-1");
    assert_eq!(body.description, "Grab some apples");
    assert_eq!(
        body.definition.steps.len(),
        quest_definition.definition.steps.len()
    );
    for step in quest_definition.definition.steps {
        assert!(body.definition.steps.iter().any(|s| s.id == step.id));
    }
    assert_eq!(
        body.definition.connections,
        quest_definition.definition.connections
    );
}

#[actix_web::test]
async fn get_quest_should_be_400() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url)
        .await
        .unwrap();

    let app = init_service(get_app_router(&Data::new(config), &Data::new(db.clone()))).await;

    let req = TestRequest::get().uri("/quests/1aaa").to_request();

    let response = call_service(&app, req).await;

    assert_eq!(response.status(), 400);
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 400);
    assert!(body.message.contains("Bad Request:"));
}

#[actix_web::test]
async fn get_quest_should_be_404() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url)
        .await
        .unwrap();

    let app = init_service(get_app_router(&Data::new(config), &Data::new(db.clone()))).await;

    let id = uuid::Uuid::new_v4().to_string();

    let req = TestRequest::get()
        .uri(format!("/quests/{}", id).as_str())
        .to_request();

    let response = call_service(&app, req).await;

    assert_eq!(response.status(), 404);
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 404);
    assert_eq!(body.message, "Not Found");
}
