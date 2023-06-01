use std::sync::Arc;

use actix_web::{put, web, HttpMessage, HttpRequest, HttpResponse};
use dcl_crypto::Address;
use derive_more::Deref;
use quests_db::{core::definitions::QuestsDatabase, Database};
use quests_protocol::definitions::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::quests::QuestError;
use crate::domain::types::ToCreateQuest;

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

    let user = {
        let extensions = req.extensions();
        if let Some(address) = extensions.get::<Address>() {
            address.to_string()
        } else {
            return HttpResponse::BadRequest().into();
        }
    };

    match update_quest_controller(db, quest_id, &quest, &user).await {
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
    creator_address: &str,
) -> Result<String, QuestError> {
    if let Err(err) = quest.is_valid() {
        return Err(QuestError::QuestValidation(err.to_string()));
    }

    match db.get_quest(&id).await {
        Ok(stored_quest) => {
            if stored_quest
                .creator_address
                .eq_ignore_ascii_case(creator_address)
            {
                db.update_quest(
                    &id,
                    &quest.to_create_quest()?,
                    &stored_quest.creator_address,
                )
                .await
                .map_err(|error| error.into())
            } else {
                Err(QuestError::NotQuestCreator)
            }
        }
        Err(err) => Err(err.into()),
    }
}
