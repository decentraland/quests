mod common;

use actix_web::test::{call_service, init_service, read_body_json, try_call_service, TestRequest};
use actix_web_lab::__reexports::serde_json;
use common::*;
use quests_db::{
    core::{
        definitions::{QuestReward, QuestRewardHook, QuestRewardItem, QuestsDatabase},
        errors::DBError,
    },
    create_quests_db_component,
};
use quests_protocol::definitions::*;
use quests_server::api::routes::{
    quests::{CreateQuestRequest, CreateQuestResponse},
    ErrorResponse,
};
use std::collections::HashMap;

#[actix_web::test]
async fn create_quest_should_be_200_without_reward() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url, true)
        .await
        .unwrap();
    let app = init_service(build_app(&config).await).await;

    let Quest {
        name,
        description,
        definition,
        image_url,
        ..
    } = quest_samples::grab_some_apples();

    let create_quest_request = CreateQuestRequest {
        name,
        definition: definition.unwrap(),
        description,
        image_url,
        reward: None,
    };

    let headers = get_signed_headers(
        create_test_identity(),
        "post",
        "/api/quests",
        serde_json::to_string(&create_quest_request)
            .unwrap()
            .as_str(),
    );

    let req = TestRequest::post()
        .uri("/api/quests")
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

    let quest_reward = db.get_quest_reward_hook(&response.id).await.unwrap_err();

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
        name,
        description,
        definition,
        image_url,
        ..
    } = quest_samples::grab_some_apples();

    let create_quest_request = CreateQuestRequest {
        name,
        definition: definition.unwrap(),
        description,
        image_url,
        reward: Some(QuestReward {
            hook: QuestRewardHook {
                webhook_url: "https://rewards.decentraland.zone/api/campaigns/649c5e38-bef8-4bd6-b13f-bd6a2bdcc096/rewards".to_string(),
                request_body: Some(HashMap::from([("campaign_key".to_string(), "value-json-webtoken".to_string()), ("beneficiary".to_string(), "{user_address}".to_string())]))
            },
            items: vec![QuestRewardItem { name: "SunGlasses".to_string(), image_link: "https://github.com/decentraland".to_string() }]
        }),
    };

    let headers = get_signed_headers(
        create_test_identity(),
        "post",
        "/api/quests",
        serde_json::to_string(&create_quest_request)
            .unwrap()
            .as_str(),
    );

    let req = TestRequest::post()
        .uri("/api/quests")
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

    let quest_reward = db.get_quest_reward_hook(&response.id).await.unwrap();

    assert_eq!(quest_reward.webhook_url, "https://rewards.decentraland.zone/api/campaigns/649c5e38-bef8-4bd6-b13f-bd6a2bdcc096/rewards");
    assert_eq!(
        quest_reward.request_body,
        Some(HashMap::from([
            (
                "campaign_key".to_string(),
                "value-json-webtoken".to_string()
            ),
            ("beneficiary".to_string(), "{user_address}".to_string())
        ]))
    );
}

#[actix_web::test]
async fn create_quest_should_be_400_quest_validation_error_missing_definition() {
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
        image_url: "".to_string(),
    };

    let headers = get_signed_headers(
        create_test_identity(),
        "post",
        "/api/quests",
        serde_json::to_string(&quest_definition).unwrap().as_str(),
    );

    let req = TestRequest::post()
        .uri("/api/quests")
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .set_json(&quest_definition)
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
async fn create_quest_should_be_400_quest_validation_error_rewards_webhook() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;
    let Quest {
        name,
        description,
        definition,
        image_url,
        ..
    } = quest_samples::grab_some_apples();

    let create_quest_request = CreateQuestRequest {
        name,
        definition: definition.unwrap(),
        description,
        image_url,
        reward: Some(QuestReward {
            hook: QuestRewardHook {
                webhook_url: "rewards.decentraland.zone/api/campaigns/649c5e38-bef8-4bd6-b13f-bd6a2bdcc096/rewards".to_string(),
                request_body: Some(HashMap::from([("campaign_key".to_string(), "value-json-webtoken".to_string()), ("beneficiary".to_string(), "{user_address}".to_string())]))
            },
            items: vec![QuestRewardItem { name: "SunGlasses".to_string(), image_link: "https://github.com/decentraland".to_string() }]
        }),
    };

    let headers = get_signed_headers(
        create_test_identity(),
        "post",
        "/api/quests",
        serde_json::to_string(&create_quest_request)
            .unwrap()
            .as_str(),
    );

    let req = TestRequest::post()
        .uri("/api/quests")
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .set_json(&create_quest_request)
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_client_error());
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 400);
    assert!(body
        .message
        .contains("Quest Validation Error: Webhook url is not valid"));
}

#[actix_web::test]
async fn create_quest_should_be_400_quest_validation_error_rewards_items() {
    let config = get_configuration().await;
    let app = init_service(build_app(&config).await).await;
    let Quest {
        name,
        description,
        definition,
        image_url,
        ..
    } = quest_samples::grab_some_apples();

    let mut create_quest_request = CreateQuestRequest {
        name,
        definition: definition.unwrap(),
        description,
        image_url,
        reward: Some(QuestReward {
            hook: QuestRewardHook {
                webhook_url: "https://rewards.decentraland.zone/api/campaigns/649c5e38-bef8-4bd6-b13f-bd6a2bdcc096/rewards".to_string(),
                request_body: Some(HashMap::from([("campaign_key".to_string(), "value-json-webtoken".to_string()), ("beneficiary".to_string(), "{user_address}".to_string())]))
            },
            items: vec![QuestRewardItem { name: "SunGlasses".to_string(), image_link: "github/decentraland".to_string() }]
        }),
    };

    let headers = get_signed_headers(
        create_test_identity(),
        "post",
        "/api/quests",
        serde_json::to_string(&create_quest_request)
            .unwrap()
            .as_str(),
    );

    let req = TestRequest::post()
        .uri("/api/quests")
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .set_json(&create_quest_request)
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_client_error());
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 400);
    assert!(body
        .message
        .contains("Quest Validation Error: Item's image link is not valid"));

    create_quest_request.reward.as_mut().unwrap().items[0].name = "A".to_string();
    create_quest_request.reward.as_mut().unwrap().items[0].image_link =
        "https://github.com/decentraland".to_string();

    let headers = get_signed_headers(
        create_test_identity(),
        "post",
        "/api/quests",
        serde_json::to_string(&create_quest_request)
            .unwrap()
            .as_str(),
    );

    let req = TestRequest::post()
        .uri("/api/quests")
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .set_json(&create_quest_request)
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_client_error());
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 400);
    assert!(body
        .message
        .contains("Quest Validation Error: Item name must be at least 3 characters"));

    create_quest_request.reward.as_mut().unwrap().items = vec![];
    let headers = get_signed_headers(
        create_test_identity(),
        "post",
        "/api/quests",
        serde_json::to_string(&create_quest_request)
            .unwrap()
            .as_str(),
    );

    let req = TestRequest::post()
        .uri("/api/quests")
        .append_header(headers[0].clone())
        .append_header(headers[1].clone())
        .append_header(headers[2].clone())
        .append_header(headers[3].clone())
        .append_header(headers[4].clone())
        .set_json(&create_quest_request)
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_client_error());
    let body: ErrorResponse = read_body_json(response).await;
    assert_eq!(body.code, 400);
    assert!(body
        .message
        .contains("Quest Validation Error: Reward items must be at least one"));
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
        image_url: "".to_string(),
        reward: None,
    };

    let req = TestRequest::post()
        .uri("/api/quests")
        .set_json(quest_definition)
        .to_request();

    match try_call_service(&app, req).await {
        Ok(res) => {
            let s = res.status();
            panic!("should fail {}", s)
        }
        Err(err) => {
            let response = err.error_response();
            assert_eq!(response.status(), 401);
        }
    }
}
