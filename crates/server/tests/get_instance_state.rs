mod common;
use actix_web::http::StatusCode;
use actix_web::test::{call_service, init_service, read_body_json, TestRequest};
pub use common::*;
use quests_db::core::definitions::{CreateQuest, QuestsDatabase};
use quests_db::create_quests_db_component;
use quests_protocol::definitions::*;
use quests_server::api::routes::quest_instances::GetInstanceStateResponse;

#[actix_web::test]
async fn get_instance_state_should_be_200() {
    let config = get_configuration(None).await;
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

    let quest_instance_id = db.start_quest(&id, "0xA").await.unwrap();

    let path = format!("/api/instances/{}/state", quest_instance_id);

    let headers = get_signed_headers(create_test_identity(), "get", &path, "");

    let req = TestRequest::get()
        .uri(&path)
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .to_request();

    let response = call_service(&app, req).await;
    assert_eq!(response.status().as_u16(), StatusCode::OK);

    let json: GetInstanceStateResponse = read_body_json(response).await;
    assert_eq!(json.state.steps_completed.len(), 0);
    assert_eq!(json.events.len(), 0)
}

#[actix_web::test]
async fn get_instance_state_should_be_403() {
    let config = get_configuration(None).await;
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
        .create_quest(&create_quest, "0x7949f9f239d1a0816ce5eb364a1f588ae9cc1ba5")
        .await
        .unwrap();

    let quest_instance_id = db.start_quest(&id, "0xA").await.unwrap();

    let path = format!("/api/instances/{}/state", quest_instance_id);

    let headers = get_signed_headers(create_test_identity(), "get", &path, "");

    let req = TestRequest::get()
        .uri(&path)
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .to_request();

    let response = call_service(&app, req).await;
    assert_eq!(response.status().as_u16(), StatusCode::FORBIDDEN);
}
