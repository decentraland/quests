use std::sync::Arc;

use actix_web::{get, web, HttpResponse};
use quests_db::{core::definitions::QuestsDatabase, Database};
use quests_protocol::quests::{Quest, QuestState};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::quests::{get_instance_state, QuestError};

#[derive(Deserialize, Serialize, ToSchema)]
pub struct GetQuestStateResponse {
    quest: Quest,
    state: QuestState,
}

#[utoipa::path(
    params(
        ("quest_instance_id" = String, description = "Quest Instance ID")
    ),
    responses(
        (status = 200, description = "Quest State", body = [GetQuestStateResponse]),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Quest not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/quests/{quest_id}/instances/{quest_instance_id}")]
pub async fn get_quest_instance_state(
    data: web::Data<Database>,
    quest_instance_id: web::Path<(String, String)>,
) -> HttpResponse {
    let db = data.into_inner();
    match get_quest_instance_state_controller(db, quest_instance_id.into_inner().1).await {
        Ok((quest, state)) => HttpResponse::Ok().json(GetQuestStateResponse { quest, state }),
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn get_quest_instance_state_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    id: String,
) -> Result<(Quest, QuestState), QuestError> {
    match db.get_quest_instance(&id).await {
        Ok(quest_instance) => get_instance_state(db.clone(), quest_instance).await,
        Err(error) => Err(error.into()),
    }
}
