use std::sync::Arc;

use actix_web::{get, web, HttpResponse};
use quests_db::{core::definitions::QuestsDatabase, core::errors::DBError, Database};
use quests_definitions::quests::{Quest, QuestDefinition};

use crate::routes::errors::{CommonError, QuestError};

#[get("/quests/{quest_id}")]
pub async fn get_quest(data: web::Data<Database>, quest_id: web::Path<String>) -> HttpResponse {
    let db = data.into_inner();
    match get_quest_controller(db, quest_id.into_inner()).await {
        Ok(quest) => HttpResponse::Ok().json(quest),
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn get_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    id: String,
) -> Result<Quest, QuestError> {
    match db.get_quest(&id).await {
        Ok(stored_quest) => {
            let definition: QuestDefinition =
                if let Ok(definition) = bincode::deserialize(&stored_quest.definition) {
                    definition
                } else {
                    return Err(QuestError::StepsDeserialization);
                };
            let quest = Quest {
                name: stored_quest.name,
                description: stored_quest.description,
                definition,
            };
            Ok(quest)
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
