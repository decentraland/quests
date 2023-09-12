use quests_db::core::definitions::{CreateQuest, StoredQuest};
use quests_protocol::definitions::*;

use crate::domain::quests::QuestError;

pub trait ToQuest {
    fn to_quest(&self, decode: bool) -> Result<Quest, QuestError>;
}

pub trait ToCreateQuest {
    fn to_create_quest(&self) -> Result<CreateQuest, QuestError>;
}

impl ToQuest for StoredQuest {
    fn to_quest(&self, decode: bool) -> Result<Quest, QuestError> {
        Ok(Quest {
            id: self.id.clone(),
            name: self.name.to_string(),
            description: self.description.to_string(),
            creator_address: self.creator_address.to_string(),
            image_url: self.image_url.to_string(),
            definition: if decode {
                Some(QuestDefinition::decode(self.definition.as_slice())?)
            } else {
                None
            },
            active: self.active,
            created_at: self.created_at as u32,
        })
    }
}

impl ToCreateQuest for Quest {
    fn to_create_quest(&self) -> Result<CreateQuest, QuestError> {
        let Quest {
            name,
            description,
            definition,
            image_url,
            ..
        } = self;

        let Some(definition) = definition else {
            return Err(QuestError::QuestValidation("Quest definition not present".to_string()));
        };

        Ok(CreateQuest {
            name,
            description,
            image_url,
            definition: definition.encode_to_vec(),
        })
    }
}
