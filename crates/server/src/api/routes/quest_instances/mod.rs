pub mod reset;

use actix_web::Scope;

pub fn services(api_scope: Scope) -> Scope {
    api_scope.service(reset::reset_quest_instance)
}
