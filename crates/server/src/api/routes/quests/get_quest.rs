use actix_web::{get, web, HttpResponse};
use quests_db::Database;
use quests_protocol::quests::Quest;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::quests;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct GetQuestResponse {
    pub quest: Quest,
}

#[utoipa::path(
    params(
        ("quest_id" = String, description = "Quest ID")
    ),
    responses(
        (status = 200, description = "Quest definition", body = GetQuestResponse),
        (status = 404, description = "Quest not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/quests/{quest_id}")]
pub async fn get_quest(data: web::Data<Database>, quest_id: web::Path<String>) -> HttpResponse {
    let db = data.into_inner();
    match quests::get_quest(db, quest_id.into_inner()).await {
        Ok(quest) => HttpResponse::Ok().json(GetQuestResponse { quest }),
        Err(err) => HttpResponse::from_error(err),
    }
}
