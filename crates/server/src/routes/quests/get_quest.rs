use std::sync::Arc;

use actix_web::{get, web, HttpResponse};
use quests_db::{
    core::definitions::{QuestsDatabase, StoredQuest},
    Database,
};
use quests_definitions::quests::Quest;

use crate::routes::errors::QuestError;

#[utoipa::path(
    responses(
        (status = 200, description = "Quest definition"),
        (status = 404, description = "Quest not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
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
    db.get_quest(&id)
        .await
        .map(|stored_quest| to_quest(&stored_quest))?
}

pub fn to_quest(stored_quest: &StoredQuest) -> Result<Quest, QuestError> {
    let definition = bincode::deserialize(&stored_quest.definition)?;
    Ok(Quest {
        name: stored_quest.name.to_string(),
        description: stored_quest.description.to_string(),
        definition,
    })
}