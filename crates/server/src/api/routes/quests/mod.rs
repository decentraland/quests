pub mod create_quest;
pub mod delete_quest;
pub mod get_quest;
pub mod get_quest_rewards;
pub mod get_quest_stats;
pub mod get_quests;
pub mod update_quest;

use actix_web::{web::ServiceConfig, HttpMessage, HttpRequest, HttpResponse};
pub use create_quest::*;
use dcl_crypto::Address;
pub use delete_quest::*;
pub use get_quest::*;
pub use get_quest_rewards::*;
pub use get_quest_stats::*;
pub use get_quests::*;
use quests_db::core::definitions::StoredQuest;
use quests_protocol::definitions::QuestDefinition;
use regex::Regex;
use serde::{Deserialize, Serialize};
pub use update_quest::*;
use utoipa::ToSchema;

pub fn services(config: &mut ServiceConfig) {
    config
        .service(get_quests)
        .service(create_quest)
        .service(update_quest)
        .service(delete_quest)
        .service(get_quest)
        .service(get_quest_rewards)
        .service(get_quest_stats);
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct ProtectedQuest {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<QuestDefinition>,
}

impl From<StoredQuest> for ProtectedQuest {
    fn from(value: StoredQuest) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            definition: None,
        }
    }
}

impl From<&StoredQuest> for ProtectedQuest {
    fn from(value: &StoredQuest) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            description: value.description.clone(),
            definition: None,
        }
    }
}

pub fn get_user_address_from_request(req: &HttpRequest) -> Result<String, HttpResponse> {
    let extensions = req.extensions();
    if let Some(address) = extensions.get::<Address>() {
        Ok(address.to_string())
    } else {
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
