use std::sync::Arc;

use crate::{api::routes::quests::get_user_address_from_request, domain::quests::QuestError};
use actix_web::{put, web, HttpRequest, HttpResponse};
use quests_db::{core::definitions::QuestsDatabase, Database};

/// Activates a quest by its ID
#[utoipa::path(
    params(
        ("quest_id" = String, description = "ID of the quest to activate")
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
    req: HttpRequest,
    data: web::Data<Database>,
    quest_id: web::Path<String>,
) -> HttpResponse {
    let db = data.into_inner();

    let user = match get_user_address_from_request(&req) {
        Ok(address) => address,
        Err(bad_request_response) => return bad_request_response,
    };

    match activate_quest_controller(db, quest_id.into_inner(), &user).await {
        Ok(()) => HttpResponse::Accepted().finish(),
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn activate_quest_controller<DB: QuestsDatabase>(
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
                match db.can_activate_quest(&id).await {
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
                }
            } else {
                Err(QuestError::NotQuestCreator)
            }
        }
        Err(err) => Err(err.into()),
    }
}
