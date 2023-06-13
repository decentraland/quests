use std::collections::HashMap;

use super::errors::DBResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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
    async fn is_active_quest(&self, quest_id: &str) -> DBResult<bool>;
    async fn has_active_quest_instance(&self, user_address: &str, quest_id: &str)
        -> DBResult<bool>;

    async fn start_quest(&self, quest_id: &str, user_address: &str) -> DBResult<String>;
    async fn abandon_quest(&self, quest_instance_id: &str) -> DBResult<String>;
    async fn get_quest_instance(&self, id: &str) -> DBResult<QuestInstance>;
    async fn is_active_quest_instance(&self, quest_instance_id: &str) -> DBResult<bool>;
    async fn get_active_user_quest_instances(
        &self,
        user_address: &str,
    ) -> DBResult<Vec<QuestInstance>>;

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

#[derive(Default, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct CreateQuest<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub definition: Vec<u8>,
}

#[derive(Default, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct StoredQuest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub definition: Vec<u8>,
    pub creator_address: String,
}

#[derive(Default, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct QuestRewardHook {
    pub webhook_url: String,
    pub request_body: Option<HashMap<String, String>>,
}

#[derive(Default, PartialEq, Serialize, Deserialize, Clone, Debug)]
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
