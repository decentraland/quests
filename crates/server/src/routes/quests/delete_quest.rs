use std::sync::Arc;

use actix_web::{delete, web, HttpResponse};
use quests_db::{core::definitions::QuestsDatabase, Database};

use crate::routes::errors::CommonError;

#[utoipa::path(
    params(
        ("quest_id" = String, description = "ID of the quest to delete")
    ),
    responses(
        (status = 202, description = "Quest deactivated"),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[delete("/quests/{quest_id}")]
pub async fn delete_quest(data: web::Data<Database>, quest_id: web::Path<String>) -> HttpResponse {
    let db = data.into_inner();
    match delete_quest_controller(db, quest_id.into_inner()).await {
        Ok(()) => HttpResponse::Accepted().finish(),
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn delete_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    id: String,
) -> Result<(), CommonError> {
    db.delete_quest(&id)
        .await
        .map(|_| ())
        .map_err(|error| error.into())
}
