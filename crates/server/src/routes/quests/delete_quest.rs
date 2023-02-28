use std::sync::Arc;

use actix_web::{delete, web, HttpResponse};
use quests_db::{core::definitions::QuestsDatabase, core::errors::DBError, Database};

use crate::routes::errors::CommonError;

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
    match db.delete_quest(&id).await {
        Ok(_) => Ok(()),
        Err(error) => match error {
            DBError::NotUUID => Err(CommonError::BadRequest(
                "the ID given is not a valid".to_string(),
            )),
            DBError::RowNotFound => Err(CommonError::NotFound),
            _ => Err(CommonError::Unknown),
        },
    }
}
