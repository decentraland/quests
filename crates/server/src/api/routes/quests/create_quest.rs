use std::sync::Arc;

use actix_web::{post, web, HttpResponse};
use derive_more::Deref;
use quests_db::{
    core::definitions::QuestsDatabase,
    Database,
};
use quests_protocol::definitions::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{api::routes::errors::CommonError, domain::{quests::QuestError, types::ToCreateQuest}};

#[derive(Serialize, ToSchema)]
pub struct CreateQuestResponse {
    id: String,
}

#[derive(Deserialize, Serialize, ToSchema, Deref)]
pub struct CreateQuestRequest(Quest);

#[utoipa::path(
    request_body = CreateQuestRequest, 
    responses(
        (status = 201, description = "Quest created", body = CreateQuestResponse),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[post("/quests")]
pub async fn create_quest(
    data: web::Data<Database>,
    quest: web::Json<CreateQuestRequest>,
) -> HttpResponse {
    let db = data.into_inner();
    match create_quest_controller(db, &quest).await {
        Ok(quest_id) => HttpResponse::Created().json(CreateQuestResponse { id: quest_id }),
        Err(error) => HttpResponse::from_error(error),
    }
}

async fn create_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    quest: &Quest,
) -> Result<String, QuestError> {
    quest
        .is_valid()
        .map_err(|error| QuestError::QuestValidation(error.to_string()))?;

    let quest_creation = quest.to_create_quest()?;
    db.create_quest(&quest_creation)
        .await
        .map_err(|_| QuestError::CommonError(CommonError::Unknown))
}

