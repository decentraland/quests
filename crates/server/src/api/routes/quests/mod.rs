use actix_web::web::ServiceConfig;

pub mod abandon_quest;
pub mod create_quest;
pub mod delete_quest;
pub mod get_all_states_by_address;
pub mod get_quest;
pub mod get_quest_state;
pub mod get_quest_stats;
pub mod get_quests;
pub mod start_quest;
pub mod update_quest;

pub use abandon_quest::*;
pub use create_quest::*;
pub use delete_quest::*;
pub use get_all_states_by_address::*;
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
        .service(abandon_quest)
        .service(get_quest)
        .service(get_quest_instance_state)
        .service(get_all_quest_states_by_user_address)
        .service(get_quest_stats);
}
