mod common;
use actix_web::test::{call_service, init_service, read_body_json, TestRequest};
pub use common::*;
use quests_db::core::definitions::{CreateQuest, QuestsDatabase};
use quests_db::create_quests_db_component;
use quests_protocol::quests::{Quest, QuestDefinition};
use quests_server::api::routes::ErrorResponse;

#[actix_web::test]
async fn deactivate_quest_should_be_200() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url, true)
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

    let app = init_service(build_app(&config).await).await;
    let req = TestRequest::delete()
        .uri(format!("/quests/{}", id).as_str())
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success());

    assert!(!db.is_active_quest(&id).await.unwrap());
}

#[actix_web::test]
async fn delete_quest_should_be_400() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;
    let req = TestRequest::delete().uri("/quests/1aab").to_request();

    let response = call_service(&app, req).await;

    assert_eq!(response.status(), 400);
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 400);
    assert!(body.message.contains("Bad Request:"));
}
