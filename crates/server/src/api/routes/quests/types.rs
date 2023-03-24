use quests_db::core::definitions::{StoredQuest, UpdateQuest};
use quests_definitions::{
    quests::{Quest, QuestDefinition},
    ProstMessage,
};

use crate::domain::quests::QuestError;

pub trait ToQuest {
    fn to_quest(&self) -> Result<Quest, QuestError>;
}

pub trait ToUpdateQuest {
    fn to_update_quest(&self) -> Result<UpdateQuest, QuestError>;
}

impl ToQuest for StoredQuest {
    fn to_quest(&self) -> Result<Quest, QuestError> {
        let definition = QuestDefinition::decode(self.definition.as_slice())?;
        Ok(Quest {
            name: self.name.to_string(),
            description: self.description.to_string(),
            definition,
        })
    }
}

impl ToUpdateQuest for Quest {
    fn to_update_quest(&self) -> Result<UpdateQuest, QuestError> {
        Ok(UpdateQuest {
            name: &self.name,
            description: &self.description,
            definition: self.definition.encode_to_vec(),
        })
    }
}
