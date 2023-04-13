use actix_web::{delete, web, HttpResponse};
use quests_db::Database;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{api::routes::errors::CommonError, domain::quests};

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct AbandonQuestRequest {
    pub user_address: String,
}

#[utoipa::path(
    request_body = AbandonQuestRequest,
    responses(
        (status = 200, description = "Quest abandoned"),
        (status = 400, description = "Bad Request"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[delete("/quests/{quest_id}/instances/{instance_id}")]
async fn abandon_quest(
    data: web::Data<Database>,
    path: web::Path<(String, String)>,
    abandon_quest: web::Json<AbandonQuestRequest>,
) -> HttpResponse {
    let db = data.into_inner();
    let (_, quest_instance_id) = path.into_inner();
    let AbandonQuestRequest { user_address } = abandon_quest.into_inner();

    match quests::abandon_quest(db, &user_address, &quest_instance_id).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(quests::QuestError::CommonError(CommonError::BadRequest(_))) => {
            HttpResponse::Forbidden().finish()
        }
        Err(err) => HttpResponse::from_error(err),
    }
}
