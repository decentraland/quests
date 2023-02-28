use std::{collections::HashMap, sync::Arc};

use actix_web::{post, web, HttpResponse};
use quests_db::{
    core::definitions::{CreateQuest, QuestsDatabase},
    Database,
};
use quests_definitions::quests::Quest;

use crate::routes::errors::{CommonError, QuestError};

#[post("/quests")]
pub async fn create_quest(data: web::Data<Database>, quest: web::Json<Quest>) -> HttpResponse {
    let db = data.into_inner();
    match create_quest_controller(db, quest.0).await {
        Ok(quest_id) => {
            let mut response_body = HashMap::new();
            response_body.insert("id", quest_id);
            HttpResponse::Created().json(response_body)
        }
        Err(error) => HttpResponse::from_error(error),
    }
}

async fn create_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    quest: Quest,
) -> Result<String, QuestError> {
    match quest.is_valid() {
        Ok(_) => {
            let quest_creation = CreateQuest {
                name: &quest.name,
                description: &quest.description,
                definition: bincode::serialize(&quest.definition).unwrap(),
            };
            db.create_quest(&quest_creation)
                .await
                .map_err(|_| QuestError::CommonError(CommonError::Unknown))
        }
        Err(error) => Err(QuestError::QuestValidation(error.to_string())),
    }
}
