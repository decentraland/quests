use actix_web::web::ServiceConfig;

mod events;
mod quests;
mod health;

pub fn services(config: &mut ServiceConfig) {
    events::services(config);
    quests::services(config);
    health::services(config);
}
