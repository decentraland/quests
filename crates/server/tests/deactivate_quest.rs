mod common;

use actix_web::test::{call_service, init_service, read_body_json, try_call_service, TestRequest};
pub use common::*;
use quests_db::core::definitions::{CreateQuest, QuestsDatabase};
use quests_db::create_quests_db_component;
use quests_protocol::definitions::*;
use quests_server::api::routes::quests::CreateQuestRequest;
use quests_server::api::routes::ErrorResponse;

#[actix_web::test]
async fn deactivate_quest_should_be_200() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url, true)
        .await
        .unwrap();

    let quest_definition = CreateQuestRequest {
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
        image_url: "".to_string(),
        definition: QuestDefinition {
            connections: vec![], // not needed for this test
            steps: vec![],       // not needed for this test
        },
        reward: None,
    };

    let create_quest = CreateQuest {
        name: &quest_definition.name,
        description: &quest_definition.description,
        image_url: &quest_definition.image_url,
        definition: vec![],
        reward: None,
    };

    let id = db
        .create_quest(&create_quest, "0x7949f9f239d1a0816ce5eb364a1f588ae9cc1bf5")
        .await
        .unwrap();

    let path = format!("/api/quests/{}", id);

    let headers = get_signed_headers(create_test_identity(), "delete", &path, "{}");

    let app = init_service(build_app(&config).await).await;
    let req = TestRequest::delete()
        .uri(&path)
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success());

    assert!(!db.is_active_quest(&id).await.unwrap());
}

#[actix_web::test]
async fn delete_quest_should_be_400() {
    let config = get_configuration().await;

    let headers = get_signed_headers(create_test_identity(), "delete", "/api/quests/1aab", "{}");

    let app = init_service(build_app(&config).await).await;
    let req = TestRequest::delete()
        .uri("/api/quests/1aab")
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .to_request();

    let response = call_service(&app, req).await;

    assert_eq!(response.status(), 400);
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 400);
    assert!(body.message.contains("Bad Request:"));
}

#[actix_web::test]
async fn delete_quest_should_be_401() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;
    let req = TestRequest::delete().uri("/api/quests/1aab").to_request();

    match try_call_service(&app, req).await {
        Ok(_) => panic!("shoudl fail"),
        Err(err) => {
            let response = err.error_response();
            assert_eq!(response.status(), 401);
        }
    }
}

#[actix_web::test]
async fn deactivate_quest_should_be_403() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url, true)
        .await
        .unwrap();

    let quest_definition = CreateQuestRequest {
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
        definition: QuestDefinition {
            connections: vec![], // not needed for this test
            steps: vec![],       // not needed for this test
        },
        image_url: "".to_string(),
        reward: None,
    };

    let create_quest = CreateQuest {
        name: &quest_definition.name,
        description: &quest_definition.description,
        image_url: &quest_definition.image_url,
        definition: vec![],
        reward: None,
    };

    let id = db.create_quest(&create_quest, "0xA").await.unwrap();

    let path = format!("/api/quests/{}", id);

    let headers = get_signed_headers(create_test_identity(), "delete", &path, "{}");

    let app = init_service(build_app(&config).await).await;
    let req = TestRequest::delete()
        .uri(&path)
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .to_request();

    let response = call_service(&app, req).await;

    assert_eq!(response.status(), 403);
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 403);
    assert!(body.message.contains("Cannot modify a quest"));
}
