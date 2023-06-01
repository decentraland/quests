mod common;
use actix_web::test::{call_service, init_service, read_body_json, try_call_service, TestRequest};
use actix_web_lab::__reexports::serde_json;
use common::*;
use quests_protocol::definitions::*;
use quests_server::api::routes::ErrorResponse;

#[actix_web::test]
async fn create_quest_should_be_200() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;

    let quest_definition = quest_samples::grab_some_apples();

    let headers = get_signed_headers(
        create_test_identity(),
        "post",
        "/quests",
        serde_json::to_string(&quest_definition).unwrap().as_str(),
    );

    let req = TestRequest::post()
        .uri("/quests")
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .set_json(quest_definition)
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success())
}

#[actix_web::test]
async fn create_quest_should_be_400_quest_validation_error() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;
    let quest_definition = Quest {
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
        definition: Some(QuestDefinition {
            connections: vec![],
            steps: vec![],
        }),
    };

    let headers = get_signed_headers(
        create_test_identity(),
        "post",
        "/quests",
        serde_json::to_string(&quest_definition).unwrap().as_str(),
    );

    let req = TestRequest::post()
        .uri("/quests")
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .set_json(quest_definition)
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_client_error());
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 400);
    assert!(body
        .message
        .contains("Quest Validation Error: Missing the definition for the quest"));
}

#[actix_web::test]
async fn create_quest_should_be_401() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;
    let quest_definition = Quest {
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
        definition: Some(QuestDefinition {
            connections: vec![],
            steps: vec![],
        }),
    };

    let req = TestRequest::post()
        .uri("/quests")
        .set_json(quest_definition)
        .to_request();

    match try_call_service(&app, req).await {
        Ok(_) => panic!("shoudl fail"),
        Err(err) => {
            let response = err.error_response();
            assert_eq!(response.status(), 401);
        }
    }
}
