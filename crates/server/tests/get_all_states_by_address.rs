mod common;
use actix_web::test::{call_service, init_service, read_body_json, TestRequest};
pub use common::*;
use quests_db::{
    core::definitions::{CreateQuest, QuestsDatabase},
    create_quests_db_component,
};
use quests_definitions::ProstMessage;
use quests_server::api::routes::quests::{GetQuestStateByUserAddressResponse, StartQuestRequest};

#[actix_web::test]
async fn get_all_states_by_user_address_should_be_200() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url)
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
        quest_id: id.clone(),
        user_address: "0xA".to_string(),
    };
    // call start quest
    let req = TestRequest::post()
        .uri("/quests/instances")
        .set_json(start_quest)
        .to_request();

    let response = call_service(&app, req).await;
    assert!(response.status().is_success());

    let req = TestRequest::get().uri("/quests/instances/0xA").to_request();

    let response = call_service(&app, req).await;
    assert!(response.status().is_success());

    let response: GetQuestStateByUserAddressResponse = read_body_json(response).await;
    assert_eq!(response.states.len(), 1);
    assert!(response.states[0]
        .1
        .current_steps
        .get(&"A".to_string())
        .is_some());
    assert_eq!(response.states[0].1.steps_left, 4);
}
