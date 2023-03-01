use super::events;
use super::health;
use super::quests;
use actix_web::web::ServiceConfig;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
        paths(
            health::live,
            quests::get_quest,
            quests::get_quests,
            quests::get_quest_state,
            quests::get_quest_stats,
            quests::start_quest,
            quests::update_quest,
            quests::create_quest,
            quests::delete_quest,
            events::add_event,
        ),
        components(
            schemas()
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
