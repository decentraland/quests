mod common;
use actix_web::test::{call_service, init_service, read_body_json, TestRequest};
use common::*;
use quests_db::core::definitions::{CreateQuest, QuestsDatabase};
use quests_db::create_quests_db_component;
use quests_definitions::quests::Quest;
use quests_definitions::ProstMessage;
use quests_server::api::routes::ErrorResponse;

#[actix_web::test]
async fn get_quest_should_be_200() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url)
        .await
        .unwrap();

    let app = init_service(build_app(&config).await).await;
    let quest_definition = quest_samples::grab_some_apples();

    let create_quest = CreateQuest {
        name: &quest_definition.name,
        description: &quest_definition.description,
        definition: quest_definition.definition.encode_to_vec(),
    };

    let id = db.create_quest(&create_quest).await.unwrap();

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
    let app = init_service(build_app(&config).await).await;
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
    let app = init_service(build_app(&config).await).await;
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
