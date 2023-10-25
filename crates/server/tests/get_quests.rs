mod common;
use actix_web::test::{call_service, init_service, read_body_json, TestRequest};
pub use common::*;
use quests_db::{
    core::definitions::{CreateQuest, QuestsDatabase},
    create_quests_db_component,
};
use quests_protocol::definitions::ProtocolMessage;
use quests_server::api::routes::{quests::GetQuestsResponse, ErrorResponse};

#[actix_web::test]
async fn get_quests_should_be_200() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;
    let db = create_quests_db_component(&config.database_url, true)
        .await
        .unwrap();

    let quest_definition = quest_samples::grab_some_pies();

    let quest = CreateQuest {
        name: &quest_definition.name,
        definition: quest_definition.definition.unwrap().encode_to_vec(),
        description: &quest_definition.description,
        image_url: &quest_definition.image_url,
        reward: None,
    };

    db.create_quest(&quest, "0xA").await.unwrap();

    let req = TestRequest::get()
        .uri("/api/quests?offset=0&limit=2")
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success());

    let body: GetQuestsResponse = read_body_json(response).await;

    assert_eq!(body.quests.len(), 1);
    assert_eq!(body.quests[0].name, quest_definition.name)
}

#[actix_web::test]
async fn get_quests_should_be_400() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;
    let req = TestRequest::get()
        .uri("/api/quests?offset=0aa")
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_client_error());

    let req = TestRequest::get().uri("/api/quests?limit=0aa").to_request();

    let response = call_service(&app, req).await;

    assert_eq!(response.status(), 400);
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 400);
    assert!(body.message.contains("Bad Request:"));

    let req = TestRequest::get()
        .uri("/api/quests?offset=10a&limit=0aa")
        .to_request();

    let response = call_service(&app, req).await;

    assert_eq!(response.status(), 400);
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 400);
    assert!(body.message.contains("Bad Request:"));
}
