mod common;
use actix_web::test::{call_service, init_service, read_body_json, TestRequest};
use common::*;
use quests_protocol::quests::{Quest, QuestDefinition};
use quests_server::api::routes::ErrorResponse;

#[actix_web::test]
async fn create_quest_should_be_200() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;

    let quest_definition = quest_samples::grab_some_apples();
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
    let app = init_service(build_app(&config).await).await;
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
