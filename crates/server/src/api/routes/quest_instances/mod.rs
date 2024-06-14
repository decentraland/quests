pub mod add_event;
pub mod get;
pub mod remove_event;
pub mod reset;
pub mod state;

use actix_web::Scope;
pub use add_event::*;
pub use get::*;
pub use remove_event::*;
pub use reset::*;
pub use state::*;

pub fn services(api_scope: Scope) -> Scope {
    api_scope
        .service(reset_quest_instance)
        .service(get_quest_instance_state)
        .service(get_quest_instance)
        .service(add_event_to_instance)
        .service(remove_event_from_instance)
}
