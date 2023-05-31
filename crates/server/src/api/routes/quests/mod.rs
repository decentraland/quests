use actix_web::web::ServiceConfig;

pub mod create_quest;
pub mod delete_quest;
pub mod get_quest;
pub mod get_quest_stats;
pub mod get_quests;
pub mod update_quest;

pub use create_quest::*;
pub use delete_quest::*;
pub use get_quest::*;
pub use get_quest_stats::*;
pub use get_quests::*;
pub use update_quest::*;

pub fn services(config: &mut ServiceConfig) {
    config
        .service(get_quests)
        .service(create_quest)
        .service(update_quest)
        .service(delete_quest)
        .service(get_quest)
        .service(get_quest_stats);
}
