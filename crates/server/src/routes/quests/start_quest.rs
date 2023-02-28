use std::sync::Arc;

use actix_web::{post, web, HttpResponse};
use log::info;
use quests_db::{core::definitions::QuestsDatabase, core::errors::DBError, Database};
use serde::{Deserialize, Serialize};

use crate::routes::errors::{CommonError, QuestError};

#[derive(Serialize, Deserialize, Debug)]
pub struct StartQuest {
    pub user_address: String,
    pub quest_id: String,
}

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
    let result = db.get_quest(&start_quest_request.quest_id).await;

    match result {
        Err(DBError::RowNotFound) => return Err(QuestError::CommonError(CommonError::NotFound)),
        Err(_) => return Err(QuestError::CommonError(CommonError::Unknown)),
        _ => info!("Quest found, can start it"),
    }

    db.start_quest(
        &start_quest_request.quest_id,
        &start_quest_request.user_address,
    )
    .await
    .map_err(|error| {
        println!("Error while starting quest: {:?}", error);
        match error {
            DBError::NotUUID => QuestError::CommonError(CommonError::BadRequest(
                "the ID given is not a valid".to_string(),
            )),
            DBError::RowNotFound => QuestError::CommonError(CommonError::NotFound),

            _ => QuestError::CommonError(CommonError::Unknown),
        }
    })
}
