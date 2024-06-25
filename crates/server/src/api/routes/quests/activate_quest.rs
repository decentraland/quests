use std::sync::Arc;

use crate::{api::middlewares::RequiredAuthUser, domain::quests::QuestError};
use actix_web::{put, web, HttpResponse};
use quests_db::{core::definitions::QuestsDatabase, Database};

/// Activates a quest by its ID
#[utoipa::path(
    params(
        ("quest_id" = String, description = "Quest UUID")
    ),
    responses(
        (status = 202, description = "Quest activated"),
        (status = 400, description = "Bad Request"),
        (status = 400, description = "Requested Quest cannot be activated because it may be prevoiusly updated and replaced with a new Quest or it may be already active"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Quest modification is forbidden"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[put("/quests/{quest_id}/activate")]
pub async fn activate_quest(
    data: web::Data<Database>,
    quest_id: web::Path<String>,
    auth_user: RequiredAuthUser,
) -> HttpResponse {
    let db = data.into_inner();

    let RequiredAuthUser { address } = auth_user;

    match activate_quest_controller(db, quest_id.into_inner(), &address).await {
        Ok(()) => HttpResponse::Accepted().finish(),
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn activate_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    id: String,
    creator_address: &str,
) -> Result<(), QuestError> {
    match db.is_quest_creator(&id, creator_address).await {
        Ok(is_creator) if !is_creator => Err(QuestError::NotQuestCreator),
        Ok(_) => match db.can_activate_quest(&id).await {
            Ok(boolean) => {
                if boolean {
                    db.activate_quest(&id)
                        .await
                        .map(|_| ())
                        .map_err(|error| error.into())
                } else {
                    Err(QuestError::QuestNotActivable)
                }
            }
            Err(err) => Err(err.into()),
        },
        Err(err) => Err(err.into()),
    }
}
