pub mod reset;
pub mod state;

use actix_web::Scope;
pub use reset::*;
pub use state::*;

pub fn services(api_scope: Scope) -> Scope {
    api_scope
        .service(reset_quest_instance)
        .service(get_quest_instance_state)
}
