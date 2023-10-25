mod common;
use actix_web::test::{call_service, init_service, read_body_json, try_call_service, TestRequest};
use actix_web_lab::__reexports::serde_json;
pub use common::*;
use quests_db::core::definitions::{CreateQuest, QuestsDatabase};
use quests_db::create_quests_db_component;
use quests_protocol::definitions::*;
use quests_protocol::quests::Coordinates;
use quests_server::api::routes::quests::{CreateQuestRequest, UpdateQuestResponse};
use quests_server::api::routes::ErrorResponse;

#[actix_web::test]
async fn update_quest_should_be_200() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url, true)
        .await
        .unwrap();

    let app = init_service(build_app(&config).await).await;
    let quest = quest_samples::grab_some_apples();

    let create_quest = CreateQuest {
        name: &quest.name,
        description: &quest.description,
        image_url: &quest.image_url,
        definition: quest.definition.as_ref().unwrap().encode_to_vec(),
        reward: None,
    };

    let id = db
        .create_quest(&create_quest, "0x7949f9f239d1a0816ce5eb364a1f588ae9cc1bf5")
        .await
        .unwrap();

    let quest_update = CreateQuestRequest {
        name: "QUEST-1_UPDATE".to_string(),
        description: "Grab some apples - Updated".to_string(),
        image_url: "".to_string(),
        definition: QuestDefinition {
            connections: vec![
                Connection::new("A-Updated", "B"),
                Connection::new("B", "C"),
                Connection::new("C", "D"),
            ],
            steps: vec![
                Step {
                    id: "A-Updated".to_string(),
                    tasks: vec![Task {
                        id: "A_1".to_string(),
                        description: "desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "desc".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: vec![Task {
                        id: "B_1".to_string(),
                        description: "desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "desc".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: vec![Task {
                        id: "C_1".to_string(),
                        description: "desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "desc".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: vec![Task {
                        id: "D_1".to_string(),
                        description: "desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "desc".to_string(),
                },
            ],
        },
        reward: None,
    };

    let path = format!("/api/quests/{}", id);

    let headers = get_signed_headers(
        create_test_identity(),
        "put",
        &path,
        &serde_json::to_string(&quest_update).unwrap(),
    );

    let req = TestRequest::put()
        .uri(&path)
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .set_json(&quest_update)
        .to_request();

    let response = call_service(&app, req).await;
    assert!(response.status().is_success());

    let old_quest_is_inactive = !db.is_active_quest(&id).await.unwrap();
    assert!(old_quest_is_inactive);

    let body: UpdateQuestResponse = read_body_json(response).await;

    let quest_updated = db.get_quest(&body.quest_id).await.unwrap();
    assert_eq!(quest_updated.name, "QUEST-1_UPDATE");
    assert_eq!(quest_updated.description, "Grab some apples - Updated");
    let definition = QuestDefinition::decode(quest_updated.definition.as_slice()).unwrap();
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
    let quest_definition = CreateQuestRequest {
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
        image_url: "".to_string(),
        definition: QuestDefinition {
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
                        description: "desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "desc".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: vec![Task {
                        id: "B_1".to_string(),
                        description: "desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "desc".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: vec![Task {
                        id: "C_1".to_string(),
                        description: "desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "desc".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: vec![Task {
                        id: "D_1".to_string(),
                        description: "desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "desc".to_string(),
                },
            ],
        },
        reward: None,
    };

    let quest_update = CreateQuestRequest {
        name: "QUEST-1_UPDATE".to_string(),
        ..quest_definition
    };

    let headers = get_signed_headers(
        create_test_identity(),
        "put",
        "/api/quests/1aa",
        &serde_json::to_string(&quest_update).unwrap(),
    );

    let req = TestRequest::put()
        .uri("/api/quests/1aa")
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
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
    let quest_definition = CreateQuestRequest {
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
        definition: QuestDefinition {
            connections: vec![], // not needed for test
            steps: vec![],       // not needed for this test
        },
        image_url: "".to_string(),
        reward: None,
    };

    let quest_update = CreateQuestRequest {
        name: "QUEST-1_UPDATE".to_string(),
        ..quest_definition
    };

    let uuid = uuid::Uuid::new_v4();

    let path = format!("/api/quests/{}", uuid);

    let headers = get_signed_headers(
        create_test_identity(),
        "put",
        &path,
        &serde_json::to_string(&quest_update).unwrap(),
    );

    let req = TestRequest::put()
        .uri(&path)
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
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
async fn update_quest_should_be_401() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;

    let quest_update = CreateQuestRequest {
        name: "QUEST-1_UPDATE".to_string(),
        description: "Grab some apples - Updated".to_string(),
        image_url: "".to_string(),
        definition: QuestDefinition {
            connections: vec![
                Connection::new("A-Updated", "B"),
                Connection::new("B", "C"),
                Connection::new("C", "D"),
            ],
            steps: vec![
                Step {
                    id: "A-Updated".to_string(),
                    tasks: vec![Task {
                        id: "A_1".to_string(),
                        description: "desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "desc".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: vec![Task {
                        id: "B_1".to_string(),
                        description: "desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "desc".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: vec![Task {
                        id: "C_1".to_string(),
                        description: "desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "desc".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: vec![Task {
                        id: "D_1".to_string(),
                        description: "desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "desc".to_string(),
                },
            ],
        },
        reward: None,
    };

    let path = format!("/api/quests/{}", uuid::Uuid::new_v4());

    let req = TestRequest::put()
        .uri(&path)
        .set_json(&quest_update)
        .to_request();

    match try_call_service(&app, req).await {
        Ok(_) => panic!("should fail"),
        Err(err) => {
            let response = err.error_response();
            assert_eq!(response.status(), 401);
        }
    }
}

#[actix_web::test]
async fn update_quest_should_be_403() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url, true)
        .await
        .unwrap();

    let app = init_service(build_app(&config).await).await;
    let quest = quest_samples::grab_some_apples();

    let create_quest = CreateQuest {
        name: &quest.name,
        description: &quest.description,
        image_url: &quest.image_url,
        definition: quest.definition.as_ref().unwrap().encode_to_vec(),
        reward: None,
    };

    let id = db.create_quest(&create_quest, "0xA").await.unwrap();

    let quest_update = CreateQuestRequest {
        name: "QUEST-1_UPDATE".to_string(),
        description: "Grab some apples - Updated".to_string(),
        image_url: "".to_string(),
        definition: QuestDefinition {
            connections: vec![
                Connection::new("A-Updated", "B"),
                Connection::new("B", "C"),
                Connection::new("C", "D"),
            ],
            steps: vec![
                Step {
                    id: "A-Updated".to_string(),
                    tasks: vec![Task {
                        id: "A_1".to_string(),
                        description: "desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "desc".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: vec![Task {
                        id: "B_1".to_string(),
                        description: "desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "desc".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: vec![Task {
                        id: "C_1".to_string(),
                        description: "desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "desc".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: vec![Task {
                        id: "D_1".to_string(),
                        description: "desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "desc".to_string(),
                },
            ],
        },
        reward: None,
    };

    let path = format!("/api/quests/{}", id);

    let headers = get_signed_headers(
        create_test_identity(),
        "put",
        &path,
        &serde_json::to_string(&quest_update).unwrap(),
    );

    let req = TestRequest::put()
        .uri(&path)
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .set_json(&quest_update)
        .to_request();

    let response = call_service(&app, req).await;

    assert_eq!(response.status(), 403);
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 403);
    assert!(body.message.contains("Cannot modify a quest"));
}
