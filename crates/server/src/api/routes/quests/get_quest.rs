use std::sync::Arc;

use actix_web::{get, web, HttpResponse};
use quests_db::{
    core::definitions::{QuestsDatabase, StoredQuest},
    Database,
};
use quests_definitions::quests::Quest;
use serde::Serialize;
use utoipa::ToSchema;

use crate::routes::errors::QuestError;

#[derive(Serialize, ToSchema)]
pub struct GetQuestResponse(Quest);

#[utoipa::path(
    params(
        ("quest_id" = String, description = "Quest ID")
    ),
    responses(
        (status = 200, description = "Quest definition", body = [GetQuestResponse]),
        (status = 404, description = "Quest not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/quests/{quest_id}")]
pub async fn get_quest(data: web::Data<Database>, quest_id: web::Path<String>) -> HttpResponse {
    let db = data.into_inner();
    match get_quest_controller(db, quest_id.into_inner()).await {
        Ok(quest) => HttpResponse::Ok().json(GetQuestResponse(quest)),
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn get_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    id: String,
) -> Result<Quest, QuestError> {
    db.get_quest(&id)
        .await
        .map(|stored_quest| stored_quest.to_quest())?
}

trait ToQuest {
    fn to_quest(&self) -> Result<Quest, QuestError>;
}

impl ToQuest for StoredQuest {
    fn to_quest(&self) -> Result<Quest, QuestError> {
        let definition = bincode::deserialize(&self.definition)?;
        Ok(Quest {
            name: self.name.to_string(),
            description: self.description.to_string(),
            definition,
        })
    }
}
