use std::sync::Arc;

use actix_web::{get, web, HttpResponse};
use quests_db::{core::definitions::QuestsDatabase, core::errors::DBError, Database};
use quests_definitions::{
    quest_state::{get_state, QuestState},
    quests::Quest,
};

use crate::routes::errors::{CommonError, QuestError};

#[get("/quests/instances/{quest_instance_id}")]
pub async fn get_quest_state(
    data: web::Data<Database>,
    quest_instance_id: web::Path<String>,
) -> HttpResponse {
    let db = data.into_inner();
    match get_quest_state_controller(db, quest_instance_id.into_inner()).await {
        Ok(quest_state) => HttpResponse::Ok().json(quest_state),
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn get_quest_state_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    id: String,
) -> Result<QuestState, QuestError> {
    match db.get_quest_instance(&id).await {
        Ok(quest_instance) => {
            let quest = db.get_quest(&quest_instance.quest_id).await;
            match quest {
                Ok(quest) => {
                    let quest = Quest {
                        name: quest.name,
                        description: quest.description,
                        definition: bincode::deserialize(&quest.definition).unwrap(), // TODO: error handling
                    };
                    let events = db.get_events(&quest_instance.id).await.unwrap();
                    let events = events
                        .iter()
                        .map(|event| bincode::deserialize(&event.event).unwrap()) // TODO: error handling
                        .collect();

                    Ok(get_state(&quest, events))
                }
                Err(_) => Err(QuestError::CommonError(CommonError::BadRequest(
                    "the quest instance ID given doesn't correspond to a valid quest".to_string(),
                ))),
            }
        }
        Err(error) => match error {
            DBError::NotUUID => Err(QuestError::CommonError(CommonError::BadRequest(
                "the ID given is not a valid".to_string(),
            ))),
            DBError::RowNotFound => Err(QuestError::CommonError(CommonError::NotFound)),
            _ => Err(QuestError::CommonError(CommonError::Unknown)),
        },
    }
}
