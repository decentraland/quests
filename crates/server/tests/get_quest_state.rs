mod common;
use actix_web::{
    http::StatusCode,
    test::{call_service, init_service, read_body_json, TestRequest},
};
pub use common::*;
use quests_db::{
    core::definitions::{CreateQuest, QuestsDatabase},
    create_quests_db_component,
};
use quests_definitions::{quest_graph::QuestGraph, quest_state::QuestState};
use quests_server::routes::quests::{StartQuestRequest, StartQuestResponse};
use uuid::Uuid;

#[actix_web::test]
async fn get_quest_state_should_be_200() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url)
        .await
        .unwrap();

    let app = init_service(build_app(&config).await).await;

    let quest_definition = quest_samples::grab_some_apples();

    let create_quest = CreateQuest {
        name: &quest_definition.name,
        description: &quest_definition.description,
        definition: bincode::serialize(&quest_definition.definition).unwrap(),
    };

    let id = db.create_quest(&create_quest).await.unwrap();

    let start_quest = StartQuestRequest {
        quest_id: id,
        user_address: "0xA".to_string(),
    };
    // call start quest
    let req = TestRequest::post()
        .uri("/quests/instances")
        .set_json(start_quest)
        .to_request();

    let response = call_service(&app, req).await;
    assert!(response.status().is_success());

    let response: StartQuestResponse = read_body_json(response).await;
    let quest_instance_id = response.quest_instance_id;

    let req = TestRequest::get()
        .uri(&format!("/quests/instances/{quest_instance_id}"))
        .to_request();

    let response = call_service(&app, req).await;
    println!("response status code {:?}", response.status());
    assert!(response.status().is_success());

    // expect response is initial state from quest
    let quest_graph: QuestGraph = (&quest_definition).into();
    let initial_state: QuestState = (&quest_graph).into();

    let response: QuestState = read_body_json(response).await;
    assert_eq!(initial_state, response);
}

#[actix_web::test]
async fn get_quest_state_should_be_400() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;

    let quest_instance_id = "some_unknown_id";
    let req = TestRequest::get()
        .uri(&format!("/quests/instances/{quest_instance_id}"))
        .to_request();

    let response = call_service(&app, req).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn get_quest_state_should_be_404() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;

    let quest_instance_id = Uuid::new_v4().to_string();
    let req = TestRequest::get()
        .uri(&format!("/quests/instances/{quest_instance_id}"))
        .to_request();

    let response = call_service(&app, req).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
