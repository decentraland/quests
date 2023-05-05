mod common;
use actix_web::test::{call_service, init_service, read_body_json, TestRequest};
pub use common::*;
use quests_db::{
    core::definitions::{CreateQuest, QuestsDatabase},
    create_quests_db_component,
};
use quests_protocol::{quest_graph::QuestGraph, quests::QuestState, ProtocolMessage};
use quests_server::api::routes::quests::{
    AbandonQuestRequest, GetQuestStateResponse, StartQuestRequest, StartQuestResponse,
};

#[actix_web::test]
async fn abandon_quest_should_be_200() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url, true)
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
    assert!(response.status().is_success());

    let response: StartQuestResponse = read_body_json(response).await;
    let quest_instance_id = response.quest_instance_id;

    let req = TestRequest::get()
        .uri(&format!("/quests/{id}/instances/{quest_instance_id}"))
        .to_request();

    let response = call_service(&app, req).await;
    println!("response status code {:?}", response.status());
    assert!(response.status().is_success());

    // expect response is initial state from quest
    let quest_graph: QuestGraph = (&quest_definition).into();
    let initial_state: QuestState = (&quest_graph).into();

    let response: GetQuestStateResponse = read_body_json(response).await;
    assert_eq!(initial_state, response.state);

    let abandon_quest = AbandonQuestRequest {
        user_address: "0xA".to_string(),
    };

    let req = TestRequest::delete()
        .uri(&format!("/quests/{id}/instances/{quest_instance_id}"))
        .set_json(abandon_quest)
        .to_request();
    let response = call_service(&app, req).await;
    assert!(response.status().is_success());
}

#[actix_web::test]
async fn abandon_quest_should_be_403() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url, true)
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
    assert!(response.status().is_success());

    let response: StartQuestResponse = read_body_json(response).await;
    let quest_instance_id = response.quest_instance_id;

    let req = TestRequest::get()
        .uri(&format!("/quests/{id}/instances/{quest_instance_id}"))
        .to_request();

    let response = call_service(&app, req).await;
    println!("response status code {:?}", response.status());
    assert!(response.status().is_success());

    // expect response is initial state from quest
    let quest_graph: QuestGraph = (&quest_definition).into();
    let initial_state: QuestState = (&quest_graph).into();

    let response: GetQuestStateResponse = read_body_json(response).await;
    assert_eq!(initial_state, response.state);

    let abandon_quest = AbandonQuestRequest {
        user_address: "0xA".to_string(),
    };

    let req = TestRequest::delete()
        .uri(&format!("/quests/{id}/instances/{quest_instance_id}"))
        .set_json(abandon_quest)
        .to_request();
    let response = call_service(&app, req).await;
    assert!(response.status().is_success());
}
