mod common;

use actix_web::{
    http::StatusCode,
    test::{call_service, init_service, read_body_json, TestRequest},
};
use common::*;
use quests_db::{
    core::definitions::{CreateQuest, QuestsDatabase},
    create_quests_db_component,
};
use quests_protocol::definitions::*;
use quests_server::api::routes::quests::GetQuestRewardResponse;

#[actix_web::test]
async fn get_quest_rewards_should_be_200() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url, true)
        .await
        .unwrap();
    let app = init_service(build_app(&config).await).await;

    let Quest {
        name,
        description,
        definition,
        ..
    } = quest_samples::grab_some_apples();

    let id = db
        .create_quest(
            &CreateQuest {
                name: &name,
                description: &description,
                definition: definition.unwrap().encode_to_vec(),
                image_url: "",
                reward: None,
            },
            "0x7949f9f239d1a0816ce5eb364a1f588ae9cc1bf5", // identity address
        )
        .await
        .unwrap();

    let reward = rewards::create_reward();
    _ = db.add_reward_hook_to_quest(&id, &reward.hook).await;
    _ = db.add_reward_items_to_quest(&id, &reward.items).await;

    let path = format!("/api/quests/{}/reward", id);
    let headers = get_signed_headers(create_test_identity(), "get", &path, "{}");

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

    let rewards: GetQuestRewardResponse = read_body_json(response).await;
    assert_eq!(rewards.items.len(), 1);
}

#[actix_web::test]
async fn quest_has_no_rewards() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url, true)
        .await
        .unwrap();
    let app = init_service(build_app(&config).await).await;

    let Quest {
        name,
        description,
        definition,
        ..
    } = quest_samples::grab_some_apples();

    let id = db
        .create_quest(
            &CreateQuest {
                name: &name,
                description: &description,
                definition: definition.unwrap().encode_to_vec(),
                image_url: "",
                reward: None,
            },
            "0x7949f9f239d1a0816ce5eb364a1f588ae9cc1bf5", // identity address
        )
        .await
        .unwrap();

    let path = format!("/api/quests/{}/reward", id);
    let headers = get_signed_headers(create_test_identity(), "get", &path, "{}");

    let req = TestRequest::get()
        .uri(&path)
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .to_request();

    let response = call_service(&app, req).await;
    assert!(!response.status().is_success());
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
