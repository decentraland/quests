use std::sync::Arc;

use actix_web::{put, web, HttpResponse};
use derive_more::Deref;
use quests_db::{core::definitions::QuestsDatabase, Database};
use quests_protocol::quests::Quest;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::quests::QuestError;

use super::types::ToUpdateQuest;

#[derive(Serialize, Deserialize, Debug, ToSchema, Deref)]
pub struct UpdateQuestRequest(Quest);

#[derive(Serialize, Deserialize, Debug, ToSchema, Deref)]
pub struct UpdateQuestResponse(Quest);

#[utoipa::path(
    request_body = UpdateQuestRequest,
    params(
        ("quest_id" = String, Path, description = "Quest ID")    
    ),
    responses(
        (status = 200, description = "Quest updated"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Quest not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[put("/quests/{quest_id}")]
pub async fn update_quest(
    data: web::Data<Database>,
    quest_id: web::Path<String>,
    quest_update: web::Json<UpdateQuestRequest>,
) -> HttpResponse {
    let db = data.into_inner();
    let quest_id = quest_id.into_inner();
    match update_quest_controller(db, quest_id, quest_update.0 .0).await {
        Ok(quest) => HttpResponse::Ok().json(UpdateQuestResponse(quest)),
        Err(error) => HttpResponse::from_error(error),
    }
}

async fn update_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    id: String,
    quest: Quest,
) -> Result<Quest, QuestError> {
    match quest.is_valid() {
        Ok(_) => db
            .update_quest(&id, &quest.to_update_quest()?)
            .await
            .map(|_| quest)
            .map_err(|error| error.into()),
        Err(error) => Err(QuestError::QuestValidation(error.to_string())),
    }
}
