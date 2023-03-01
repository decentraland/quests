use std::{collections::HashMap, sync::Arc};

use actix_web::{post, web, HttpResponse};
use quests_db::{
    core::definitions::{CreateQuest, QuestsDatabase},
    Database,
};
use quests_definitions::quests::Quest;

use crate::routes::errors::{CommonError, QuestError};

#[utoipa::path(
    responses(
        (status = 201, description = "Quest created"),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[post("/quests")]
pub async fn create_quest(data: web::Data<Database>, quest: web::Json<Quest>) -> HttpResponse {
    let db = data.into_inner();
    match create_quest_controller(db, quest.0).await {
        Ok(quest_id) => {
            let response_body = HashMap::from([("id", quest_id)]);
            HttpResponse::Created().json(response_body)
        }
        Err(error) => HttpResponse::from_error(error),
    }
}

async fn create_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    quest: Quest,
) -> Result<String, QuestError> {
    quest
        .is_valid()
        .map_err(|error| QuestError::QuestValidation(error.to_string()))?;

    let quest_creation = to_create_quest(&quest)?;
    db.create_quest(&quest_creation)
        .await
        .map_err(|_| QuestError::CommonError(CommonError::Unknown))
}

fn to_create_quest(quest: &Quest) -> Result<CreateQuest, QuestError> {
    Ok(CreateQuest {
        name: &quest.name,
        description: &quest.description,
        definition: bincode::serialize(&quest.definition)?,
    })
}
