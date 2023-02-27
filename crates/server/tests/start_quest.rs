mod common;
use actix_web::test::*;
use common::quest_samples::*;
use common::*;
use quests_db::core::definitions::{CreateQuest, QuestsDatabase};
use quests_db::create_quests_db_component;
use quests_server::routes::quests::StartQuest;

#[actix_web::test]
async fn can_start_a_quest() {
    let config = get_configuration().await;
    let db = create_quests_db_component(&config.database_url)
        .await
        .unwrap();

    let app = init_service(build_app(&config).await).await;

    let quest_definition = simple_quest();

    let create_quest = CreateQuest {
        name: &quest_definition.name,
        description: &quest_definition.description,
        definition: bincode::serialize(&quest_definition.definition).unwrap(),
    };

    let id = db.create_quest(&create_quest).await.unwrap();

    let start_quest = StartQuest {
        quest_id: id,
        user_address: "0xA".to_string(),
    };
    // call start quest
    let req = TestRequest::post()
        .uri("/quests/instances")
        .set_json(start_quest)
        .to_request();

    let response = call_service(&app, req).await;

    assert!(response.status().is_success())
}
