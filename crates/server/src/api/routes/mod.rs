use actix_web::web::ServiceConfig;

mod api_doc;
pub mod errors;
mod events;
mod health;
pub mod quests;

pub use errors::{query_extractor_config, ErrorResponse};

pub(crate) fn services(config: &mut ServiceConfig) {
    api_doc::services(config);
    events::services(config);
    quests::services(config);
    health::services(config);
}
