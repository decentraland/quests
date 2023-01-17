use actix_web::web::ServiceConfig;

mod events;
mod health;
mod quests;

pub fn services(config: &mut ServiceConfig) {
    events::services(config);
    quests::services(config);
    health::services(config);
}
