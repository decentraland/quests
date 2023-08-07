use actix_web::web::{self, ServiceConfig};

mod api_doc;
pub mod creators;
pub mod errors;
mod health;
pub mod quests;

pub use errors::{query_extractor_config, ErrorResponse};

pub(crate) fn services(config: &mut ServiceConfig) {
    api_doc::services(config);

    let api_scope = web::scope("/api");
    let api_scope = quests::services(api_scope);
    let api_scope = creators::services(api_scope);
    config.service(api_scope);

    health::services(config);
}
