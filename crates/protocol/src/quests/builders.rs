use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{definitions::*, quests::Quest};

const LOCATION: &str = "LOCATION";
const JUMP: &str = "JUMP";
const NPC_INTERACTION: &str = "NPC_INTERACTION";
const CUSTOM: &str = "CUSTOM";
pub(crate) const EMOTE: &str = "EMOTE";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Coordinates {
    pub x: isize,
    pub y: isize,
}

impl Coordinates {
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }
}

impl Action {
    pub fn location(coords: Coordinates) -> Self {
        let parameters = HashMap::from_iter([
            ("x".to_string(), coords.x.to_string()),
            ("y".to_string(), coords.y.to_string()),
        ]);

        Self {
            r#type: LOCATION.to_string(),
            parameters,
        }
    }

    pub fn jump(coords: Coordinates) -> Self {
        let parameters = HashMap::from_iter([
            ("x".to_string(), coords.x.to_string()),
            ("y".to_string(), coords.y.to_string()),
        ]);

        Self {
            r#type: JUMP.to_string(),
            parameters,
        }
    }

    pub fn emote(coords: Coordinates, emote_id: &str) -> Self {
        let parameters = HashMap::from_iter([
            ("x".to_string(), coords.x.to_string()),
            ("y".to_string(), coords.y.to_string()),
            ("id".to_string(), emote_id.to_string()),
        ]);

        Self {
            r#type: EMOTE.to_string(),
            parameters,
        }
    }

    pub fn custom(id: &str) -> Self {
        let parameters = HashMap::from_iter([("id".to_string(), id.to_string())]);
        Self {
            r#type: CUSTOM.to_string(),
            parameters,
        }
    }

    pub fn npc_interaction(npc_id: &str) -> Self {
        let parameters = HashMap::from_iter([("npc_id".to_string(), npc_id.to_string())]);

        Self {
            r#type: NPC_INTERACTION.to_string(),
            parameters,
        }
    }
}

impl StartQuestResponse {
    fn response(response: start_quest_response::Response) -> Self {
        Self {
            response: Some(response),
        }
    }
    pub fn accepted() -> Self {
        Self::response(start_quest_response::Response::Accepted(
            start_quest_response::Accepted {},
        ))
    }

    pub fn invalid_quest() -> Self {
        Self::response(start_quest_response::Response::InvalidQuest(
            InvalidQuest {},
        ))
    }

    pub fn not_uuid_error() -> Self {
        Self::response(start_quest_response::Response::NotUuidError(NotUuid {}))
    }

    pub fn quest_already_started() -> Self {
        Self::response(start_quest_response::Response::QuestAlreadyStarted(
            QuestAlreadyStarted {},
        ))
    }

    pub fn internal_server_error() -> Self {
        Self::response(start_quest_response::Response::InternalServerError(
            InternalServerError {},
        ))
    }
}

impl AbortQuestResponse {
    pub fn accepted() -> Self {
        Self {
            response: Some(abort_quest_response::Response::Accepted(
                abort_quest_response::Accepted {},
            )),
        }
    }

    pub fn not_found_quest_instance() -> Self {
        Self {
            response: Some(abort_quest_response::Response::NotFoundQuestInstance(
                NotFoundQuestInstance {},
            )),
        }
    }

    pub fn not_owner() -> Self {
        Self {
            response: Some(abort_quest_response::Response::NotOwner(NotOwner {})),
        }
    }

    pub fn not_uuid_error() -> Self {
        Self {
            response: Some(abort_quest_response::Response::NotUuidError(NotUuid {})),
        }
    }

    pub fn internal_server_error() -> Self {
        Self {
            response: Some(abort_quest_response::Response::InternalServerError(
                InternalServerError {},
            )),
        }
    }
}

impl EventResponse {
    pub fn accepted(event_id: uuid::Uuid) -> Self {
        Self {
            response: Some(event_response::Response::AcceptedEventId(
                event_id.to_string(),
            )),
        }
    }

    pub fn ignored() -> Self {
        Self {
            response: Some(event_response::Response::IgnoredEvent(IgnoredEvent {})),
        }
    }

    pub fn internal_server_error() -> Self {
        Self {
            response: Some(event_response::Response::InternalServerError(
                InternalServerError {},
            )),
        }
    }
}

impl GetAllQuestsResponse {
    pub fn ok(instances: Vec<QuestInstance>) -> Self {
        Self {
            response: Some(get_all_quests_response::Response::Quests(Quests {
                instances,
            })),
        }
    }

    pub fn internal_server_error() -> Self {
        Self {
            response: Some(get_all_quests_response::Response::InternalServerError(
                InternalServerError {},
            )),
        }
    }
}

impl GetQuestDefinitionResponse {
    pub fn ok(quest: Quest) -> Self {
        Self {
            response: Some(get_quest_definition_response::Response::Quest(quest)),
        }
    }

    pub fn internal_server_error() -> Self {
        Self {
            response: Some(
                get_quest_definition_response::Response::InternalServerError(
                    InternalServerError {},
                ),
            ),
        }
    }
}
