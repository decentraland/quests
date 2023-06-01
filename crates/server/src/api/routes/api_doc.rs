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
                quests::get_quest_stats,
                quests::update_quest,
                quests::create_quest,
                quests::delete_quest,
        ),
        components(
                schemas(
                        quests::create_quest::CreateQuestRequest,
                        quests::create_quest::CreateQuestResponse,
                        quests::get_quest::GetQuestResponse,
                        quests::get_quests::GetQuestsResponse,
                        quests::update_quest::UpdateQuestRequest,
                        quests::update_quest::UpdateQuestResponse,
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
