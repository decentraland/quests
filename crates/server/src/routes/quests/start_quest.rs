use std::sync::Arc;

use actix_web::{post, web, HttpResponse};
use quests_db::{core::definitions::QuestsDatabase, Database};
use serde::{Deserialize, Serialize};

use crate::routes::errors::QuestError;

#[derive(Serialize, Deserialize, Debug)]
pub struct StartQuest {
    pub user_address: String,
    pub quest_id: String,
}

#[utoipa::path()]
#[post("/quests/instances")]
async fn start_quest(
    data: web::Data<Database>,
    start_quest: web::Json<StartQuest>,
) -> HttpResponse {
    let db = data.into_inner();
    let start_quest = start_quest.into_inner();

    match start_quest_controller(db, start_quest).await {
        Ok(quest_instance_id) => HttpResponse::Ok().json(quest_instance_id),
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn start_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    start_quest_request: StartQuest,
) -> Result<String, QuestError> {
    db.get_quest(&start_quest_request.quest_id)
        .await
        .map_err(|err| -> QuestError { err.into() })?;

    db.start_quest(
        &start_quest_request.quest_id,
        &start_quest_request.user_address,
    )
    .await
    .map_err(|error| error.into())
}
