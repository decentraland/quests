use std::sync::Arc;

use actix_web::{put, web, HttpResponse};
use derive_more::Deref;
use quests_db::{core::definitions::QuestsDatabase, Database};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::middlewares::RequiredAuthUser;
use crate::domain::quests::QuestError;
use crate::domain::types::ToCreateQuest;

use super::CreateQuestRequest;

#[derive(Serialize, Deserialize, Debug, ToSchema, Deref)]
pub struct UpdateQuestRequest(CreateQuestRequest);

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct UpdateQuestResponse {
    pub quest_id: String,
}

/// Update a quest.
///
/// Returns the ID of the updated quest
#[utoipa::path(
    request_body = UpdateQuestRequest,
    params(
        ("quest_id" = String, Path, description = "Quest ID")
    ),
    responses(
        (status = 200, description = "Quest updated", body = UpdateQuestResponse),
        (status = 400, description = "Bad Request"),
        (status = 400, description = "Requested Quest was previously updated and replaced with a new Quest"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Quest not found"),
        (status = 403, description = "Quest modification is forbidden"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[put("/quests/{quest_id}")]
pub async fn update_quest(
    data: web::Data<Database>,
    quest_id: web::Path<String>,
    quest_update: web::Json<UpdateQuestRequest>,
    auth_user: RequiredAuthUser,
) -> HttpResponse {
    let db = data.into_inner();
    let quest_id = quest_id.into_inner();
    let quest = quest_update.into_inner();

    let RequiredAuthUser { address } = auth_user;

    match update_quest_controller(db, &quest_id, &quest, &address).await {
        Ok(quest_id) => HttpResponse::Ok().json(UpdateQuestResponse { quest_id }),
        Err(error) => HttpResponse::from_error(error),
    }
}

async fn update_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    id: &str,
    quest: &CreateQuestRequest,
    creator_address: &str,
) -> Result<String, QuestError> {
    quest.is_valid()?;

    match db.is_quest_creator(id, creator_address).await {
        Ok(is_creator) if !is_creator => Err(QuestError::NotQuestCreator),
        Ok(_) => {
            if !db.is_updatable(id).await? {
                return Err(QuestError::QuestIsNotUpdatable);
            }
            db.update_quest(id, &quest.to_create_quest()?, creator_address)
                .await
                .map_err(|error| {
                    log::error!("Couldn't update quest: {error}");
                    error.into()
                })
        }
        Err(err) => Err(err.into()),
    }
}
