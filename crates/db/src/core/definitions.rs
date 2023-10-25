use std::collections::HashMap;

use super::errors::{DBError, DBResult};
use crate::{date_time_to_unix, parse_uuid_to_str};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, Row};
use utoipa::ToSchema;

#[async_trait]
pub trait QuestsDatabase: Send + Sync + CloneDatabase {
    async fn ping(&self) -> bool;

    async fn create_quest(&self, quest: &CreateQuest, creator_address: &str) -> DBResult<String>;
    async fn update_quest(
        &self,
        previous_quest_id: &str,
        quest: &CreateQuest,
        creator_address: &str,
    ) -> DBResult<String>;
    async fn deactivate_quest(&self, id: &str) -> DBResult<String>;
    async fn get_quest(&self, id: &str) -> DBResult<StoredQuest>;
    async fn get_active_quests(&self, offset: i64, limit: i64) -> DBResult<Vec<StoredQuest>>;
    async fn get_quests_by_creator_id(
        &self,
        creator_id: &str,
        offset: i64,
        limit: i64,
    ) -> DBResult<Vec<StoredQuest>>;
    async fn is_active_quest(&self, quest_id: &str) -> DBResult<bool>;
    async fn has_active_quest_instance(&self, user_address: &str, quest_id: &str)
        -> DBResult<bool>;

    async fn start_quest(&self, quest_id: &str, user_address: &str) -> DBResult<String>;
    async fn abandon_quest_instance(&self, quest_instance_id: &str) -> DBResult<String>;
    async fn complete_quest_instance(&self, quest_instance_id: &str) -> DBResult<String>;
    async fn is_completed_instance(&self, quest_instance_id: &str) -> DBResult<bool>;

    async fn get_quest_instance(&self, id: &str) -> DBResult<QuestInstance>;
    async fn is_active_quest_instance(&self, quest_instance_id: &str) -> DBResult<bool>;
    async fn get_active_user_quest_instances(
        &self,
        user_address: &str,
    ) -> DBResult<Vec<QuestInstance>>;

    async fn get_quest_instances_by_quest_id(
        &self,
        quest_id: &str,
    ) -> DBResult<(Vec<QuestInstance>, Vec<QuestInstance>)>;

    async fn add_event(&self, event: &AddEvent, quest_instance_id: &str) -> DBResult<()>;
    async fn get_events(&self, quest_instance_id: &str) -> DBResult<Vec<Event>>;

    async fn add_reward_hook_to_quest(
        &self,
        quest_id: &str,
        reward: &QuestRewardHook,
    ) -> DBResult<()>;
    async fn get_quest_reward_hook(&self, quest_id: &str) -> DBResult<QuestRewardHook>;
    async fn add_reward_items_to_quest(
        &self,
        quest_id: &str,
        items: &[QuestRewardItem],
    ) -> DBResult<()>;
    async fn get_quest_reward_items(&self, quest_id: &str) -> DBResult<Vec<QuestRewardItem>>;

    async fn can_activate_quest(&self, quest_id: &str) -> DBResult<bool>;
    async fn activate_quest(&self, quest_id: &str) -> DBResult<bool>;

    async fn is_updatable(&self, quest_id: &str) -> DBResult<bool>;

    async fn get_old_quest_versions(&self, quest_id: &str) -> DBResult<Vec<String>>;
}

#[derive(Default, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct AddEvent<'a> {
    pub id: String,
    pub user_address: &'a str,
    pub event: Vec<u8>,
}

#[derive(Default, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    pub id: String,
    pub user_address: String,
    pub quest_instance_id: String,
    pub timestamp: i64,
    pub event: Vec<u8>,
}

#[derive(Default, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct QuestInstance {
    pub id: String,
    pub quest_id: String,
    pub user_address: String,
    pub start_timestamp: i64,
}

impl TryFrom<PgRow> for QuestInstance {
    type Error = DBError;
    fn try_from(value: PgRow) -> Result<Self, Self::Error> {
        Ok(QuestInstance {
            id: parse_uuid_to_str(
                value
                    .try_get("id")
                    .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
            ),
            quest_id: parse_uuid_to_str(
                value
                    .try_get("quest_id")
                    .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
            ),
            user_address: value
                .try_get("user_address")
                .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
            start_timestamp: date_time_to_unix(
                value
                    .try_get("start_timestamp")
                    .map_err(|err| DBError::RowCorrupted(Box::new(err)))?,
            ),
        })
    }
}

#[derive(Default, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct CreateQuest<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub image_url: &'a str,
    pub definition: Vec<u8>,
    pub reward: Option<QuestReward>,
}

#[derive(Default, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct StoredQuest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub definition: Vec<u8>,
    pub creator_address: String,
    pub image_url: String,
    pub active: bool,
    pub created_at: i64,
}

#[derive(Default, PartialEq, Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct QuestReward {
    pub hook: QuestRewardHook,
    pub items: Vec<QuestRewardItem>,
}

#[derive(Default, PartialEq, Serialize, Deserialize, Clone, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuestRewardHook {
    pub webhook_url: String,
    pub request_body: Option<HashMap<String, String>>,
}

#[derive(Default, PartialEq, Serialize, Deserialize, Clone, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuestRewardItem {
    pub name: String,
    pub image_link: String,
}

pub trait CloneDatabase {
    fn clone_db(&self) -> Box<dyn QuestsDatabase>;
}

impl<T> CloneDatabase for T
where
    T: QuestsDatabase + Clone + 'static,
{
    fn clone_db(&self) -> Box<dyn QuestsDatabase> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn QuestsDatabase> {
    fn clone(&self) -> Self {
        (**self).clone_db()
    }
}
