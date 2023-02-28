use std::sync::Arc;

use actix_web::{put, web, HttpResponse};
use quests_db::{
    core::definitions::{QuestsDatabase, UpdateQuest},
    core::errors::DBError,
    Database,
};
use quests_definitions::quests::Quest;

use crate::routes::errors::{CommonError, QuestError};

#[put("/quests/{quest_id}")]
pub async fn update_quest(
    data: web::Data<Database>,
    quest_id: web::Path<String>,
    quest_update: web::Json<Quest>,
) -> HttpResponse {
    let db = data.into_inner();
    let quest_id = quest_id.into_inner();
    match update_quest_controller(db, quest_id, quest_update.0).await {
        Ok(quest) => HttpResponse::Ok().json(quest),
        Err(error) => HttpResponse::from_error(error),
    }
}

async fn update_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    id: String,
    quest: Quest,
) -> Result<Quest, QuestError> {
    match quest.is_valid() {
        Ok(_) => {
            let update = UpdateQuest {
                name: &quest.name,
                description: &quest.description,
                definition: bincode::serialize(&quest.definition).unwrap(),
            };
            match db.update_quest(&id, &update).await {
                Ok(_) => Ok(quest),
                Err(error) => match error {
                    DBError::NotUUID => Err(QuestError::CommonError(CommonError::BadRequest(
                        "the ID given is not a valid".to_string(),
                    ))),
                    DBError::RowNotFound => Err(QuestError::CommonError(CommonError::NotFound)),
                    _ => Err(QuestError::CommonError(CommonError::NotFound)),
                },
            }
        }
        Err(error) => Err(QuestError::QuestValidation(error.to_string())),
    }
}
