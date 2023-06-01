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
use quests_db::core::definitions::StoredQuest;
use quests_protocol::definitions::QuestDefinition;
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
