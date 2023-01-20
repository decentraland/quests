pub mod errors;
pub mod ops;
use async_trait::async_trait;
use errors::DBResult;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait QuestsDatabase: Send + Sync + CloneDatabase {
    async fn ping(&self) -> bool;

    async fn create_quest(&self, quest: &CreateQuest) -> DBResult<String>;
    async fn update_quest(&self, quest_id: &str, quest: &UpdateQuest) -> DBResult<()>;
    async fn get_quest(&self, id: &str) -> DBResult<Quest>;
    async fn delete_quest(&self, id: &str) -> DBResult<()>;
    async fn start_quest(&self, quest_id: &str, user_address: &str) -> DBResult<String>;

    async fn get_quest_instance(&self, id: &str) -> DBResult<QuestInstance>;
    async fn get_user_quest_instances(&self, user_address: &str) -> DBResult<Vec<QuestInstance>>;

    async fn add_event(&self, event: &AddEvent, quest_instance_id: &str) -> DBResult<()>;
    async fn get_events(&self, quest_instance_id: &str) -> DBResult<Vec<Event>>;
}

#[derive(Default, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct AddEvent<'a> {
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
pub struct UpdateQuest<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub definition: Vec<u8>,
}

#[derive(Default, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct CreateQuest<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub definition: Vec<u8>,
}

#[derive(Default, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct Quest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub definition: Vec<u8>,
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
