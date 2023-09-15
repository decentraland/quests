use std::sync::Arc;

use crate::{api::routes::quests::get_user_address_from_request, domain::quests::QuestError};
use actix_web::{delete, web, HttpRequest, HttpResponse};
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
    req: HttpRequest,
    data: web::Data<Database>,
    quest_id: web::Path<String>,
) -> HttpResponse {
    let db = data.into_inner();

    let user = match get_user_address_from_request(&req) {
        Ok(address) => address,
        Err(bad_request_response) => return bad_request_response,
    };

    match delete_quest_controller(db, quest_id.into_inner(), &user).await {
        Ok(()) => HttpResponse::Accepted().finish(),
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn delete_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    id: String,
    creator_address: &str,
) -> Result<(), QuestError> {
    match db.get_quest(&id).await {
        Ok(stored_quest) => {
            if stored_quest
                .creator_address
                .eq_ignore_ascii_case(creator_address)
            {
                match db.is_active_quest(&id).await {
                    Ok(result) => {
                        if result {
                            db.deactivate_quest(&id)
                                .await
                                .map(|_| ())
                                .map_err(|error| error.into())
                        } else {
                            Err(QuestError::QuestIsCurrentlyDeactivated)
                        }
                    }
                    Err(err) => Err(err.into()),
                }
            } else {
                Err(QuestError::NotQuestCreator)
            }
        }
        Err(err) => Err(err.into()),
    }
}
