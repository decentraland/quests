pub mod activate_quest;
pub mod create_quest;
pub mod delete_quest;
pub mod get_quest;
pub mod get_quest_reward;
pub mod get_quest_stats;
pub mod get_quest_updates;
pub mod get_quests;
pub mod update_quest;

pub use super::creators::get_quests_by_creator_id::get_quests_by_creator_id;
pub use activate_quest::*;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Scope};
pub use create_quest::*;
use dcl_crypto::Address;
pub use delete_quest::*;
pub use get_quest::*;
pub use get_quest_reward::*;
pub use get_quest_stats::*;
pub use get_quest_updates::*;
pub use get_quests::*;
use regex::Regex;
pub use update_quest::*;

pub fn services(api_scope: Scope) -> Scope {
    api_scope
        .service(get_quests)
        .service(create_quest)
        .service(update_quest)
        .service(delete_quest)
        .service(get_quest)
        .service(get_quest_reward)
        .service(get_quest_stats)
        .service(activate_quest)
        .service(get_quest_updates)
}

pub fn get_user_address_from_request(req: &HttpRequest) -> Result<String, HttpResponse> {
    let extensions = req.extensions();
    if let Some(address) = extensions.get::<Address>() {
        Ok(address.to_string().to_lowercase())
    } else {
        log::error!("No Address");
        Err(HttpResponse::BadRequest().body("Bad Request"))
    }
}

#[allow(clippy::invalid_regex)]
pub fn is_url(url: &str) -> bool {
    let regex = Regex::new(
        r"https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()!@:%_\+.~#?&\/\/=]*)",
    )
    .unwrap();
    regex.is_match(url)
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_be_valid_url() {
        let url = "https://www.google.com";
        assert!(super::is_url(url));

        let url = "http://google.com";
        assert!(super::is_url(url));
    }

    #[test]
    fn should_be_not_valid_url() {
        let url = "https:/google.com";
        assert!(!super::is_url(url));
    }
}
