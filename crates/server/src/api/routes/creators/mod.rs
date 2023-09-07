pub mod get_quests_by_creator_id;

use actix_web::Scope;
pub use get_quests_by_creator_id::*;

pub fn services(api_scope: Scope) -> Scope {
    api_scope.service(get_quests_by_creator_id)
}
