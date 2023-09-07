use super::creators;
use super::health;
use super::quests;
use actix_web::web::ServiceConfig;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
        info(title = "Quests API", description = "Quests API for content creators and progress tracking"),
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
                        quests::get_quest::GetQuestResponse,
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
            (name = "events", description = "Events endpoints.")
        ),
)]
struct ApiDoc;

pub(crate) fn services(config: &mut ServiceConfig) {
    config
        .service(SwaggerUi::new("/api/doc/{_:.*}").url("/api-doc/openapi.json", ApiDoc::openapi()));
}
