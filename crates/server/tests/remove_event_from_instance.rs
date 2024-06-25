mod common;
use actix_web::http::StatusCode;
use actix_web::test::{call_service, init_service, TestRequest};
pub use common::*;
use quests_db::core::definitions::{AddEvent, CreateQuest, QuestsDatabase};
use quests_db::create_quests_db_component;
use quests_protocol::definitions::*;
use uuid::Uuid;

#[actix_web::test]
async fn remove_event_from_instance_should_be_200() {
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

    let event_id = Uuid::new_v4().to_string();
    db.add_event(
        &AddEvent {
            id: event_id.clone(),
            user_address: "0xA",
            event: vec![],
        },
        &quest_instance_id,
    )
    .await
    .unwrap();

    let path = format!("/api/instances/{}/events/{}", quest_instance_id, event_id);

    let headers = get_signed_headers(create_test_identity(), "delete", &path, "");

    let req = TestRequest::delete()
        .uri(&path)
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .to_request();

    let response = call_service(&app, req).await;
    assert_eq!(response.status().as_u16(), StatusCode::NO_CONTENT);
}

#[actix_web::test]
async fn remove_event_from_instance_should_be_403() {
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

    let path = format!(
        "/api/instances/{}/events/{}",
        quest_instance_id,
        Uuid::new_v4()
    );

    let headers = get_signed_headers(create_test_identity(), "delete", &path, "");

    let req = TestRequest::delete()
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

#[actix_web::test]
async fn remove_event_from_instance_should_be_404() {
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

    let path = format!(
        "/api/instances/{}/events/{}",
        Uuid::new_v4(),
        Uuid::new_v4()
    );

    let headers = get_signed_headers(create_test_identity(), "delete", &path, "");

    let req = TestRequest::delete()
        .uri(&path)
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .to_request();

    let response = call_service(&app, req).await;
    assert_eq!(response.status().as_u16(), StatusCode::NOT_FOUND);

    let quest_instance_id = db.start_quest(&id, "0xA").await.unwrap();

    let path = format!(
        "/api/instances/{}/events/{}",
        quest_instance_id,
        Uuid::new_v4()
    );

    let headers = get_signed_headers(create_test_identity(), "delete", &path, "");

    let req = TestRequest::delete()
        .uri(&path)
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .to_request();

    let response = call_service(&app, req).await;
    assert_eq!(response.status().as_u16(), StatusCode::NOT_FOUND);
}
