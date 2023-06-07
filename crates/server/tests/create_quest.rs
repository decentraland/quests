mod common;
use actix_web::test::{call_service, init_service, read_body_json, try_call_service, TestRequest};
use actix_web_lab::__reexports::serde_json;
use common::*;
use quests_db::{
    core::{
        definitions::{QuestReward, QuestsDatabase},
        errors::DBError,
    },
    create_quests_db_component,
};
use quests_protocol::definitions::*;
use quests_server::api::routes::{
    quests::{CreateQuestRequest, CreateQuestResponse},
    ErrorResponse,
};

#[actix_web::test]
async fn create_quest_should_be_200_without_reward() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url, true)
        .await
        .unwrap();
    let app = init_service(build_app(&config).await).await;

    let Quest {
        id: _,
        name,
        description,
        definition,
    } = quest_samples::grab_some_apples();

    let create_quest_request = CreateQuestRequest {
        name,
        definition: definition.unwrap(),
        description,
        reward: None,
    };

    let headers = get_signed_headers(
        create_test_identity(),
        "post",
        "/quests",
        serde_json::to_string(&create_quest_request)
            .unwrap()
            .as_str(),
    );

    let req = TestRequest::post()
        .uri("/quests")
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .set_json(create_quest_request)
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success());

    let response: CreateQuestResponse = read_body_json(response).await;

    let quest_reward = db.get_quest_reward(&response.id).await.unwrap_err();

    assert!(matches!(quest_reward, DBError::RowNotFound));
}

#[actix_web::test]
async fn create_quest_should_be_200_with_reward() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url, true)
        .await
        .unwrap();
    let app = init_service(build_app(&config).await).await;

    let Quest {
        id: _,
        name,
        description,
        definition,
    } = quest_samples::grab_some_apples();

    let campaign_id = uuid::Uuid::new_v4();

    let create_quest_request = CreateQuestRequest {
        name,
        definition: definition.unwrap(),
        description,
        reward: Some(QuestReward {
            campaign_id: campaign_id.to_string(),
            auth_key: "token".to_string(),
        }),
    };

    let headers = get_signed_headers(
        create_test_identity(),
        "post",
        "/quests",
        serde_json::to_string(&create_quest_request)
            .unwrap()
            .as_str(),
    );

    let req = TestRequest::post()
        .uri("/quests")
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .set_json(create_quest_request)
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success());

    let response: CreateQuestResponse = read_body_json(response).await;

    let quest_reward = db.get_quest_reward(&response.id).await.unwrap();

    assert_eq!(quest_reward.auth_key, "token");
    assert_eq!(quest_reward.campaign_id, campaign_id.to_string());
}

#[actix_web::test]
async fn create_quest_should_be_400_quest_validation_error() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;
    let quest_definition = CreateQuestRequest {
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
        definition: QuestDefinition {
            connections: vec![],
            steps: vec![],
        },
        reward: None,
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
    let quest_definition = CreateQuestRequest {
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
        definition: QuestDefinition {
            connections: vec![],
            steps: vec![],
        },
        reward: None,
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
