use actix_web::web::ServiceConfig;

mod create_quest;
mod delete_quest;
mod get_quest;
mod get_quest_state;
mod get_quest_stats;
mod get_quests;
mod start_quest;
mod update_quest;

pub use create_quest::*;
pub use delete_quest::*;
pub use get_quest::*;
pub use get_quest_state::*;
pub use get_quest_stats::*;
pub use get_quests::*;
pub use start_quest::*;
pub use update_quest::*;

pub fn services(config: &mut ServiceConfig) {
    config
        .service(get_quests)
        .service(create_quest)
        .service(update_quest)
        .service(delete_quest)
        .service(start_quest)
        .service(get_quest)
        .service(get_quest_state)
        .service(get_quest_stats);
}
