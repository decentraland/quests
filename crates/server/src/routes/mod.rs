use actix_web::web::ServiceConfig;

mod errors;
mod events;
mod health;
mod quests;

use errors::CommonError;

pub fn services(config: &mut ServiceConfig) {
    events::services(config);
    quests::services(config);
    health::services(config);
}
