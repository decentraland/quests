use super::events;
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
                quests::get_quest_instance_state,
                quests::get_quest_stats,
                quests::start_quest,
                quests::abandon_quest,
                quests::update_quest,
                quests::create_quest,
                quests::delete_quest,
                events::add_event,
        ),
        components(
                schemas(
                        quests::create_quest::CreateQuestRequest,
                        quests::create_quest::CreateQuestResponse,
                        quests::get_quest::GetQuestResponse,
                        quests::get_quest_state::GetQuestStateResponse,
                        quests::get_quests::GetQuestsResponse,
                        quests::start_quest::StartQuestRequest,
                        quests::start_quest::StartQuestResponse,
                        quests::abandon_quest::AbandonQuestRequest,
                        quests::update_quest::UpdateQuestRequest,
                        quests::update_quest::UpdateQuestResponse,
                        events::AddEventResponse,
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
