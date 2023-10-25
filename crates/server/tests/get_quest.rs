mod common;
use actix_web::test::{call_service, init_service, read_body_json, TestRequest};
use actix_web_lab::__reexports::serde_json;
use common::*;
use quests_db::core::definitions::{CreateQuest, QuestsDatabase};
use quests_db::create_quests_db_component;
use quests_protocol::definitions::*;
use quests_server::api::routes::quests::GetQuestResponse;
use quests_server::api::routes::ErrorResponse;

#[actix_web::test]
async fn get_quest_with_defintiions_should_be_200() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url, true)
        .await
        .unwrap();

    let app = init_service(build_app(&config).await).await;
    let quest_definition = quest_samples::grab_some_apples();

    let create_quest = CreateQuest {
        name: &quest_definition.name,
        description: &quest_definition.description,
        image_url: &quest_definition.image_url,
        definition: quest_definition
            .definition
            .as_ref()
            .unwrap()
            .encode_to_vec(),
        reward: None,
    };

    let id = db
        .create_quest(&create_quest, "0x7949f9f239d1a0816ce5eb364a1f588ae9cc1bf5")
        .await
        .unwrap();

    let path = format!("/api/quests/{}", id);

    let headers = get_signed_headers(
        create_test_identity(),
        "get",
        &path,
        serde_json::to_string(&quest_definition).unwrap().as_str(),
    );

    let req = TestRequest::get()
        .uri(&path)
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success());
    let GetQuestResponse { quest } = read_body_json(response).await;
    assert_eq!(quest.name, "QUEST-1");
    assert_eq!(quest.description, "Grab some apples");
    assert_eq!(
        quest.definition.as_ref().unwrap().steps.len(),
        quest_definition.definition.as_ref().unwrap().steps.len()
    );
    for step in &quest_definition.definition.as_ref().unwrap().steps {
        assert!(quest
            .definition
            .as_ref()
            .unwrap()
            .steps
            .iter()
            .any(|s| s.id == step.id));
    }
    assert_eq!(
        quest.definition.unwrap().connections,
        quest_definition.definition.unwrap().connections
    );
}

#[actix_web::test]
async fn get_quest_without_defintiions_should_be_200() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url, true)
        .await
        .unwrap();

    let app = init_service(build_app(&config).await).await;
    let quest_definition = quest_samples::grab_some_apples();

    let create_quest = CreateQuest {
        name: &quest_definition.name,
        description: &quest_definition.description,
        image_url: &quest_definition.image_url,
        definition: quest_definition
            .definition
            .as_ref()
            .unwrap()
            .encode_to_vec(),
        reward: None,
    };

    let id = db
        .create_quest(&create_quest, "0x7949f9f239d1a0816ce5eb364a1f588ae9cc1bf5")
        .await
        .unwrap();

    let path = format!("/api/quests/{}", id);

    let req = TestRequest::get().uri(&path).to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success());
    let GetQuestResponse { quest } = read_body_json(response).await;
    assert_eq!(quest.name, "QUEST-1");
    assert_eq!(quest.description, "Grab some apples");
    assert!(quest.definition.is_none());
}

#[actix_web::test]
async fn get_quest_should_be_400() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;
    let req = TestRequest::get().uri("/api/quests/1aaa").to_request();

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
        .uri(format!("/api/quests/{}", id).as_str())
        .to_request();

    let response = call_service(&app, req).await;

    assert_eq!(response.status(), 404);
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 404);
    assert_eq!(body.message, "Not Found");
}
