use std::sync::Arc;

use actix_web::{get, web, HttpResponse};
use quests_db::{core::definitions::QuestsDatabase, Database};
use quests_definitions::quest_state::{get_state, QuestState};

use crate::routes::errors::{CommonError, QuestError};

use super::get_quest::to_quest;

#[utoipa::path(
    responses(
        (status = 200, description = "Quest State"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Quest not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/quests/instances/{quest_instance_id}")]
pub async fn get_quest_state(
    data: web::Data<Database>,
    quest_instance_id: web::Path<String>,
) -> HttpResponse {
    let db = data.into_inner();
    match get_quest_state_controller(db, quest_instance_id.into_inner()).await {
        Ok(quest_state) => HttpResponse::Ok().json(quest_state),
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn get_quest_state_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    id: String,
) -> Result<QuestState, QuestError> {
    match db.get_quest_instance(&id).await {
        Ok(quest_instance) => {
            let quest = db.get_quest(&quest_instance.quest_id).await;
            match quest {
                Ok(stored_quest) => {
                    let quest = to_quest(&stored_quest)?;
                    let stored_events = db.get_events(&quest_instance.id).await?;

                    let events = stored_events
                        .iter()
                        .map(|event| bincode::deserialize(&event.event))
                        .collect::<Result<Vec<_>, _>>()?;

                    Ok(get_state(&quest, events))
                }
                Err(_) => Err(QuestError::CommonError(CommonError::BadRequest(
                    "the quest instance ID given doesn't correspond to a valid quest".to_string(),
                ))),
            }
        }
        Err(error) => Err(error.into()),
    }
}
