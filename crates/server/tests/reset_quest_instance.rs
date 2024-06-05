mod common;
use actix_web::http::StatusCode;
use actix_web::test::{call_service, init_service, TestRequest};
pub use common::*;
use quests_db::core::definitions::{AddEvent, CreateQuest, QuestsDatabase};
use quests_db::create_quests_db_component;
use quests_protocol::definitions::*;

#[actix_web::test]
async fn reset_quest_instance_should_be_204() {
    let config = get_configuration().await;
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
    db.add_event(
        &AddEvent {
            id: uuid::Uuid::new_v4().to_string(),
            user_address: "0xA",
            event: vec![0],
        },
        &quest_instance_id,
    )
    .await
    .unwrap();
    db.complete_quest_instance(&quest_instance_id)
        .await
        .unwrap();
    let is_instance_completed = db.is_completed_instance(&quest_instance_id).await.unwrap();
    assert!(is_instance_completed);
    let events = db.get_events(&quest_instance_id).await.unwrap();
    assert_eq!(events.len(), 1);

    let path = format!("/api/instances/{}/reset", quest_instance_id);

    let headers = get_signed_headers(create_test_identity(), "patch", &path, "");

    let req = TestRequest::patch()
        .uri(&path)
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .to_request();

    let response = call_service(&app, req).await;
    assert_eq!(response.status().as_u16(), StatusCode::NO_CONTENT);

    let is_instance_still_completed = db.is_completed_instance(&quest_instance_id).await.unwrap();
    assert!(!is_instance_still_completed);
    let events = db.get_events(&quest_instance_id).await.unwrap();
    assert_eq!(events.len(), 0)
}

#[actix_web::test]
async fn reset_quest_instance_should_be_403() {
    let config = get_configuration().await;
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
    db.add_event(
        &AddEvent {
            id: uuid::Uuid::new_v4().to_string(),
            user_address: "0xA",
            event: vec![0],
        },
        &quest_instance_id,
    )
    .await
    .unwrap();
    db.complete_quest_instance(&quest_instance_id)
        .await
        .unwrap();

    let path = format!("/api/instances/{}/reset", quest_instance_id);

    let headers = get_signed_headers(create_test_identity(), "patch", &path, "");

    let req = TestRequest::patch()
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
