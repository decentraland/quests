use actix_web::web::ServiceConfig;

mod errors;
mod events;
mod health;
mod quests;

use errors::CommonError;
pub use errors::{query_extractor_config, ErrorResponse};

pub(crate) fn services(config: &mut ServiceConfig) {
    events::services(config);
    quests::services(config);
    health::services(config);
}
