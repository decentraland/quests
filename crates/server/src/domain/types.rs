use quests_db::core::definitions::{CreateQuest, StoredQuest};
use quests_protocol::{
    quests::{Quest, QuestDefinition},
    ProtocolMessage,
};

use crate::domain::quests::QuestError;

pub trait ToQuest {
    fn to_quest(&self) -> Result<Quest, QuestError>;
}

pub trait ToCreateQuest {
    fn to_create_quest(&self) -> Result<CreateQuest, QuestError>;
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

impl ToCreateQuest for Quest {
    fn to_create_quest(&self) -> Result<CreateQuest, QuestError> {
        Ok(CreateQuest {
            name: &self.name,
            description: &self.description,
            definition: self.definition.encode_to_vec(),
        })
    }
}
