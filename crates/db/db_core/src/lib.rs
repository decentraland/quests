pub mod errors;
pub mod ops;
use async_trait::async_trait;
use errors::DBResult;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait QuestsDatabase: Send + Sync + CloneDatabase {
    fn ping(&self) -> DBResult<bool>;

    fn create_quest(&self, quest: &CreateQuest) -> DBResult<()>;
    fn update_quest(&self, quest_id: &str, quest: &UpdateQuest) -> DBResult<()>;
    fn get_quest(&self, id: &str) -> DBResult<()>;
    fn delete_quest(&self, id: &str) -> DBResult<()>;
    fn start_quest(&self, quest_id: &str, user_address: &str) -> DBResult<()>;

    fn get_quest_instance(&self, id: &str) -> DBResult<QuestInstance>;
    fn get_user_quest_instances(&self, user_address: &str) -> DBResult<Vec<QuestInstance>>;

    fn add_event(&self, event: &AddEvent, quest_instance_id: &str) -> DBResult<()>;
    fn get_events(&self, quest_instance_id: &str) -> DBResult<Vec<Event>>;
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
    pub timestamp: usize,
    pub event: Vec<u8>,
}

#[derive(Default, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct QuestInstance {
    pub id: String,
    pub quest_id: String,
    pub user_address: String,
    pub start_timestamp: usize,
}

#[derive(Default, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct UpdateQuest<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub definition: Vec<u8>,
}

#[derive(Default, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct CreateQuest<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub description: &'a str,
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
