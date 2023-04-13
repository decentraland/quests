mod common;
use actix_web::http::StatusCode;
use actix_web::test::*;
use common::*;
use quests_db::core::definitions::{CreateQuest, QuestsDatabase};
use quests_db::create_quests_db_component;
use quests_protocol::ProtocolMessage;
use quests_server::api::routes::quests::StartQuestRequest;
use uuid::Uuid;

#[actix_web::test]
async fn start_quest_should_be_200() {
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

    let start_quest = StartQuestRequest {
        user_address: "0xA".to_string(),
    };
    // call start quest
    let req = TestRequest::post()
        .uri(&format!("/quests/{id}/instances"))
        .set_json(start_quest)
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success())
}
#[actix_web::test]
async fn start_quest_should_be_400() {
    // should not be able to start a quest that is inactive
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
    db.deactivate_quest(&id).await.unwrap();

    let start_quest = StartQuestRequest {
        user_address: "0xA".to_string(),
    };
    // call start quest
    let req = TestRequest::post()
        .uri(&format!("/quests/{id}/instances"))
        .set_json(start_quest)
        .to_request();

    let response = call_service(&app, req).await;

    assert!(!response.status().is_success());
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // should not be able to start a quest that doesn't exist
    let id = Uuid::new_v4().to_string();

    let start_quest = StartQuestRequest {
        user_address: "0xA".to_string(),
    };
    // call start quest
    let req = TestRequest::post()
        .uri(&format!("/quests/{id}/instances"))
        .set_json(start_quest)
        .to_request();

    let response = call_service(&app, req).await;
    assert!(!response.status().is_success());
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
