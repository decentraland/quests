use std::sync::Arc;

use crate::domain::{quests::QuestError, types::ToQuest};
use actix_web::{get, web, HttpResponse};
use quests_db::{core::definitions::QuestsDatabase, Database};
use quests_protocol::definitions::Quest;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct GetQuestsQuery {
    offset: Option<i64>,
    limit: Option<i64>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GetQuestsResponse {
    pub quests: Vec<Quest>,
}

/// Get quests.
///
/// Quests don't include their definition.
#[utoipa::path(
    params(
        ("query" = GetQuestsQuery, Query, description = "Offset and limit params")
    ),
    responses(
        (status = 200, description = "Quest Definition", body = GetQuestsResponse),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Quest not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/quests")]
pub async fn get_quests(
    db: web::Data<Database>,
    query: web::Query<GetQuestsQuery>,
) -> HttpResponse {
    let db = db.into_inner();

    match get_quests_controller(db, query.offset.unwrap_or(0), query.limit.unwrap_or(50)).await {
        Ok(quests) => HttpResponse::Ok().json(GetQuestsResponse { quests }),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn get_quests_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    offset: i64,
    limit: i64,
) -> Result<Vec<Quest>, QuestError> {
    match db.get_active_quests(offset, limit).await {
        Ok(stored_quests) => {
            let mut quests = vec![];
            for stored_quest in stored_quests {
                match stored_quest.to_quest(false) {
                    Ok(quest) => quests.push(quest),
                    Err(err) => return Err(err),
                }
            }
            Ok(quests)
        }
        Err(err) => Err(err.into()),
    }
}
