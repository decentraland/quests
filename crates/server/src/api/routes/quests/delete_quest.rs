use std::sync::Arc;

use crate::{api::middlewares::RequiredAuthUser, domain::quests::QuestError};
use actix_web::{delete, web, HttpResponse};
use quests_db::{core::definitions::QuestsDatabase, Database};

/// Deactivate a quest
#[utoipa::path(
    params(
        ("quest_id" = String, description = "ID of the quest to deactivate")
    ),
    responses(
        (status = 202, description = "Quest deactivated"),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Quest modification is forbidden"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[delete("/quests/{quest_id}")]
pub async fn delete_quest(
    data: web::Data<Database>,
    quest_id: web::Path<String>,
    auth_user: RequiredAuthUser,
) -> HttpResponse {
    let db = data.into_inner();

    let RequiredAuthUser { address } = auth_user;

    match delete_quest_controller(db, &quest_id.into_inner(), &address).await {
        Ok(()) => HttpResponse::Accepted().finish(),
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn delete_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    id: &str,
    creator_address: &str,
) -> Result<(), QuestError> {
    match db.is_quest_creator(id, creator_address).await {
        Ok(is_creator) if !is_creator => Err(QuestError::NotQuestCreator),
        Ok(_) => match db.is_active_quest(id).await {
            Ok(result) => {
                if result {
                    db.deactivate_quest(id)
                        .await
                        .map(|_| ())
                        .map_err(|error| error.into())
                } else {
                    Err(QuestError::QuestIsCurrentlyDeactivated)
                }
            }
            Err(err) => Err(err.into()),
        },
        Err(err) => Err(err.into()),
    }
}
