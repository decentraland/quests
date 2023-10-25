mod common;

use actix_web::test::{call_service, init_service, read_body_json, try_call_service, TestRequest};
use common::*;
use quests_db::{
    core::definitions::{CreateQuest, QuestsDatabase},
    create_quests_db_component,
};
use quests_protocol::definitions::*;
use quests_server::api::routes::quests::GetQuestStatsResponse;

#[actix_web::test]
async fn get_quest_stats_should_be_200() {
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

    db.start_quest(&id, "0xA").await.unwrap();
    let instance_id = db.start_quest(&id, "0xB").await.unwrap();
    db.start_quest(&id, "0xC").await.unwrap();

    let headers = get_signed_headers(
        create_test_identity(),
        "get",
        format!("/api/quests/{}/stats", id).as_str(),
        "{}",
    );

    let req = TestRequest::get()
        .uri(format!("/api/quests/{}/stats", id).as_str())
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .to_request();

    db.abandon_quest_instance(&instance_id).await.unwrap();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success());

    let response: GetQuestStatsResponse = read_body_json(response).await;

    assert_eq!(response.active_players, 2);
    assert_eq!(response.abandoned, 1);
    assert_eq!(response.started_in_last_24_hours, 2);
    assert_eq!(response.completed, 0);
}

#[actix_web::test]
async fn get_quest_stats_should_be_403() {
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
            "0xB",
        )
        .await
        .unwrap();

    db.start_quest(&id, "0xA").await.unwrap();

    let headers = get_signed_headers(
        create_test_identity(),
        "get",
        format!("/api/quests/{}/stats", id).as_str(),
        "{}",
    );

    let req = TestRequest::get()
        .uri(format!("/api/quests/{}/stats", id).as_str())
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .to_request();

    let response = call_service(&app, req).await;

    assert_eq!(response.status(), 403)
}

#[actix_web::test]
async fn get_quest_stats_should_be_401() {
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
            "0xB",
        )
        .await
        .unwrap();

    let req = TestRequest::get()
        .uri(format!("/api/quests/{}/stats", id).as_str())
        .to_request();

    match try_call_service(&app, req).await {
        Ok(_) => panic!("should fail"),
        Err(err) => {
            let res = err.error_response();
            assert_eq!(res.status(), 401)
        }
    }
}
