use quests_db::core::definitions::{CreateQuest, StoredQuest};
use quests_protocol::definitions::*;

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
            id: self.id.clone(),
            name: self.name.to_string(),
            description: self.description.to_string(),
            definition: Some(definition),
        })
    }
}

impl ToCreateQuest for Quest {
    fn to_create_quest(&self) -> Result<CreateQuest, QuestError> {
        let Quest {
            name,
            description,
            definition,
            ..
        } = self;

        let Some(definition) = definition else {
            return Err(QuestError::QuestValidation("Quest definition not present".to_string()));
        };

        Ok(CreateQuest {
            name,
            description,
            definition: definition.encode_to_vec(),
        })
    }
}
