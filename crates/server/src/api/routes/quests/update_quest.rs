use std::sync::Arc;

use actix_web::{put, web, HttpResponse};
use derive_more::Deref;
use quests_db::{core::definitions::QuestsDatabase, Database};
use quests_protocol::quests::Quest;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::quests::QuestError;

use super::types::ToCreateQuest;

#[derive(Serialize, Deserialize, Debug, ToSchema, Deref)]
pub struct UpdateQuestRequest(Quest);

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct UpdateQuestResponse {
    pub quest: Quest,
    pub quest_id: String,
}

#[utoipa::path(
    request_body = UpdateQuestRequest,
    params(
        ("quest_id" = String, Path, description = "Quest ID")    
    ),
    responses(
        (status = 200, description = "Quest updated", body = UpdateQuestResponse),
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
    let quest = quest_update.into_inner();
    match update_quest_controller(db, quest_id, &quest).await {
        Ok(quest_id) => HttpResponse::Ok().json(UpdateQuestResponse {
            quest_id,
            quest: quest.0,
        }),
        Err(error) => HttpResponse::from_error(error),
    }
}

async fn update_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    id: String,
    quest: &Quest,
) -> Result<String, QuestError> {
    match quest.is_valid() {
        Ok(_) => db
            .update_quest(&id, &quest.to_create_quest()?)
            .await
            .map_err(|error| error.into()),
        Err(error) => Err(QuestError::QuestValidation(error.to_string())),
    }
}
