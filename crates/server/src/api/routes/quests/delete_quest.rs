use std::sync::Arc;

use crate::domain::quests::QuestError;
use actix_web::{delete, web, HttpMessage, HttpRequest, HttpResponse};
use dcl_crypto::Address;
use quests_db::{core::definitions::QuestsDatabase, Database};

#[utoipa::path(
    params(
        ("quest_id" = String, description = "ID of the quest to delete")
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

    let user = {
        let extensions = req.extensions();
        if let Some(address) = extensions.get::<Address>() {
            address.to_string()
        } else {
            // almost impossible branch
            return HttpResponse::BadRequest().body("Bad Request");
        }
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
                db.deactivate_quest(&id)
                    .await
                    .map(|_| ())
                    .map_err(|error| error.into())
            } else {
                Err(QuestError::NotQuestCreator)
            }
        }
        Err(err) => Err(err.into()),
    }
}
