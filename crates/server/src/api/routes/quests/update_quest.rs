use std::sync::Arc;

use actix_web::{put, web, HttpRequest, HttpResponse};
use derive_more::Deref;
use quests_db::{core::definitions::QuestsDatabase, Database};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::routes::quests::get_user_address_from_request;
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
    req: HttpRequest,
    data: web::Data<Database>,
    quest_id: web::Path<String>,
    quest_update: web::Json<UpdateQuestRequest>,
) -> HttpResponse {
    let db = data.into_inner();
    let quest_id = quest_id.into_inner();
    let quest = quest_update.into_inner();

    let user = match get_user_address_from_request(&req) {
        Ok(address) => address,
        Err(bad_request_response) => return bad_request_response,
    };

    match update_quest_controller(db, quest_id, &quest, &user).await {
        Ok(quest_id) => HttpResponse::Ok().json(UpdateQuestResponse { quest_id }),
        Err(error) => HttpResponse::from_error(error),
    }
}

async fn update_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    id: String,
    quest: &CreateQuestRequest,
    creator_address: &str,
) -> Result<String, QuestError> {
    quest.is_valid()?;

    match db.get_quest(&id).await {
        Ok(stored_quest) => {
            if stored_quest
                .creator_address
                .eq_ignore_ascii_case(creator_address)
            {
                if !db.is_updatable(&id).await? {
                    return Err(QuestError::QuestIsNotUpdatable);
                }
                db.update_quest(
                    &id,
                    &quest.to_create_quest()?,
                    &stored_quest.creator_address,
                )
                .await
                .map_err(|error| {
                    log::error!("Couldn't update quest: {error}");
                    error.into()
                })
            } else {
                Err(QuestError::NotQuestCreator)
            }
        }
        Err(err) => Err(err.into()),
    }
}
