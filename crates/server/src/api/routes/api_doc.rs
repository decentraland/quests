use super::creators;
use super::health;
use super::quest_instances;
use super::quests;
use actix_web::web::ServiceConfig;
use actix_web_lab::__reexports::serde_json::{json, to_value};
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
                quests::get_quest_instances,
                creators::get_quests_by_creator_id,
                quest_instances::reset_quest_instance,
                quest_instances::get_quest_instance_state,
                quest_instances::add_event_to_instance,
                quest_instances::get_quest_instance,
                quest_instances::remove_event_from_instance,
        ),
        components(
                schemas(
                        quests::create_quest::CreateQuestRequest,
                        quests::create_quest::CreateQuestResponse,
                        quests::get_quest::GetQuestResponse,
                        quests::get_quests::GetQuestsQuery,
                        quests::get_quests::GetQuestsResponse,
                        quests::update_quest::UpdateQuestRequest,
                        quests::update_quest::UpdateQuestResponse,
                        quests::get_quest_reward::GetQuestRewardResponse,
                        quests::get_quest_stats::GetQuestStatsResponse,
                        quests::get_quest_updates::GetQuestUpdatesResponse,
                        creators::get_quests_by_creator_id::GetCreatorQuestsResponse,
                        quests_protocol::definitions::Quest,
                        quests_protocol::definitions::QuestDefinition,
                        quests_protocol::definitions::Step,
                        quests_protocol::definitions::Task,
                        quests_protocol::definitions::Action,
                        quests_protocol::definitions::Connection,
                        quests_protocol::definitions::QuestState,
                        quests_protocol::definitions::StepContent,
                        quests_protocol::definitions::EventRequest,
                        quests_protocol::definitions::Task,
                        quests_db::core::definitions::QuestReward,
                        quests_db::core::definitions::QuestRewardHook,
                        quests_db::core::definitions::QuestRewardItem,
                        quests_db::core::definitions::Event,
                        quests_db::core::definitions::QuestInstance,
                        quest_instances::state::GetInstanceStateResponse,
                        quests::get_instances::GetQuestInstancesResponse,
                        quests::get_instances::GetQuestInstancesQuery,
                        quest_instances::add_event::AddEventToInstancePayload,
                        quest_instances::add_event::AddEventToInstanceResponse,
                        quest_instances::get::GetQuestInstanceResponse,
                )
        ),
        tags(
            (name = "quests", description = "Quests endpoints."),
            (name = "creators", description = "Creators endpoints."),
            (name = "quest_instances", description = "Quest Instances endpoints.")
        ),
)]
struct ApiDoc;

pub(crate) fn services(config: &mut ServiceConfig) {
    let html = include_str!("../../../docs/index.html");

    let mut api_doc = ApiDoc::openapi();
    let mut api_json = to_value(&mut api_doc).unwrap();
    if let Some(info) = api_json["info"].as_object_mut() {
        info.insert(
            "x-logo".to_string(),
            json!({
                "url": "https://cryptologos.cc/logos/decentraland-mana-logo.png",
            }),
        );
    }

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
