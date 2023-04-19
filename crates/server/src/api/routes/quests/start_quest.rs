use crate::domain::quests;
use actix_web::{post, web, HttpResponse};
use quests_db::Database;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct StartQuestRequest {
    pub user_address: String,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct StartQuestResponse {
    pub quest_instance_id: String,
}

#[utoipa::path(
    request_body = StartQuestRequest,
    responses(
        (status = 200, description = "Quest started", body = StartQuestResponse),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Quest not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[post("/quests/{quest_id}/instances")]
async fn start_quest(
    data: web::Data<Database>,
    path: web::Path<String>,
    start_quest: web::Json<StartQuestRequest>,
) -> HttpResponse {
    let db = data.into_inner();
    let quest_id = path.into_inner();
    let StartQuestRequest { user_address } = start_quest.into_inner();

    match quests::start_quest(db, &user_address, &quest_id).await {
        Ok(quest_instance_id) => HttpResponse::Ok().json(StartQuestResponse { quest_instance_id }),
        Err(err) => HttpResponse::from_error(err),
    }
}
