use crate::domain::quests::QuestError;

use actix_web::{get, web, HttpResponse};
use quests_db::{core::definitions::QuestsDatabase, Database};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct GetQuestUpdatesResponse {
    pub updates: Vec<String>,
}

/// Get a quest updates
/// Returns the IDs of the old quests
#[utoipa::path(
    params(
        ("quest_id" = String, description = "Quest ID")
    ),
    responses(
        (status = 200, description = "IDs of the old quests", body = GetQuestUpdatesResponse),
        (status = 404, description = "No Update"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/quests/{quest_id}/updates")]
pub async fn get_quest_updates(
    data: web::Data<Database>,
    quest_id: web::Path<String>,
) -> HttpResponse {
    let db = data.into_inner();

    let quest_id = quest_id.into_inner();

    match db.get_old_quest_versions(&quest_id).await {
        Ok(ids) => {
            if ids.is_empty() {
                return HttpResponse::NotFound().finish();
            }
            HttpResponse::Accepted().json(GetQuestUpdatesResponse { updates: ids })
        }
        Err(err) => {
            let err: QuestError = err.into();
            HttpResponse::from_error(err)
        }
    }
}
