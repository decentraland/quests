use super::creators;
use super::health;
use super::quests;
use actix_web::web::ServiceConfig;
use actix_web_lab::__reexports::serde_json;
use actix_web_lab::__reexports::serde_json::json;
use utoipa::openapi::Object;
use utoipa::openapi::ObjectBuilder;
use utoipa::OpenApi;
use utoipa_redoc::Redoc;
use utoipa_redoc::Servable;

#[derive(OpenApi)]
#[openapi(
        info(
                title = "Quests API",
                description = "Quests API for content creators and progress tracking",
                license(name = "Apache 2.0", url = "http://www.apache.org/licenses/LICENSE-2.0.html"),
        ),
        paths(
                health::live,
                quests::get_quest,
                quests::get_quests,
                quests::get_quest_reward,
                quests::get_quest_stats,
                quests::update_quest,
                quests::create_quest,
                quests::delete_quest,
                quests::get_quest_stats,
                quests::activate_quest,
                quests::get_quest_updates,
                creators::get_quests_by_creator_id,
        ),
        components(
                schemas(
                        quests::create_quest::CreateQuestRequest,
                        quests::create_quest::CreateQuestResponse,
                        quests::create_quest::QuestReward,
                        quests::get_quest::GetQuestResponse,
                        quests::get_quests::GetQuestsQuery,
                        quests::get_quests::GetQuestsResponse,
                        quests::update_quest::UpdateQuestRequest,
                        quests::update_quest::UpdateQuestResponse,
                        quests::get_quest_reward::GetQuestRewardResponse,
                        quests::get_quest_stats::GetQuestStatsResponse,
                        quests::get_quest_updates::GetQuestUpdatesResponse,
                        creators::get_quests_by_creator_id::GetCreatorQuestsResponse
                )
        ),
        tags(
            (name = "quests", description = "Quests endpoints."),
            (name = "creators", description = "Creators endpoints.")
        ),
)]
struct ApiDoc;

pub(crate) fn services(config: &mut ServiceConfig) {
    let html = include_str!("../../../docs/index.html");

    let mut api_doc = ApiDoc::openapi();
    let mut api_json = serde_json::to_value(&mut api_doc).unwrap();
    api_json["info"].as_object_mut().map(|info| {
        info.insert(
            "x-logo".to_string(),
            json!({
                "url": "https://cryptologos.cc/logos/decentraland-mana-logo.png",
            }),
        );
    });
    println!("api_json:{}", api_json.to_string());

    config.service(
        Redoc::with_url_and_config("/api/docs", api_json, || {
            json!(
                {
                    "sideNavStyle": "path-only",
                    "theme": { "colors": { "primary": { "main": "#32329f"}}}
                }
            )
        })
        .custom_html(html),
    );
}

pub fn quests_schema() -> Object {
    ObjectBuilder::new().build()
}

pub fn quests_vec_schema() -> Object {
    ObjectBuilder::new().build()
}

pub fn object_schema() -> Object {
    ObjectBuilder::new().build()
}
