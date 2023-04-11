use std::sync::Arc;

use actix_web::{get, web, HttpResponse};
use quests_db::{core::definitions::QuestsDatabase, Database};
use quests_protocol::quests::Quest;
use serde::Serialize;
use utoipa::ToSchema;

use crate::domain::quests::QuestError;

use super::types::ToQuest;

#[derive(Serialize, ToSchema)]
pub struct GetQuestResponse {
    quest: Quest,
}

#[utoipa::path(
    params(
        ("quest_id" = String, description = "Quest ID")
    ),
    responses(
        (status = 200, description = "Quest definition", body = GetQuestResponse),
        (status = 404, description = "Quest not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/quests/{quest_id}")]
pub async fn get_quest(data: web::Data<Database>, quest_id: web::Path<String>) -> HttpResponse {
    let db = data.into_inner();
    match get_quest_controller(db, quest_id.into_inner()).await {
        Ok(quest) => HttpResponse::Ok().json(GetQuestResponse { quest }),
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
