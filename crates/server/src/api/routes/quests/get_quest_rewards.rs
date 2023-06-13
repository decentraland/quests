use crate::{api::routes::errors::CommonError, domain::quests::QuestError};
use actix_web::{get, web, HttpResponse};
use quests_db::{
    core::definitions::{QuestRewardItem, QuestsDatabase},
    Database,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct QuestRewards {
    pub items: Vec<QuestRewardItem>,
}

#[utoipa::path(
    responses(
        (status = 200, description = "Quest Rewards", body = QuestRewards),
        (status = 404, description = "Not found rewards"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/quests/{quest_id}/rewards")]
pub async fn get_quest_rewards(
    quest_id: web::Path<String>,
    db: web::Data<Database>,
) -> HttpResponse {
    let quest_id = quest_id.into_inner();
    let db = db.into_inner();

    match get_quest_rewards_controller(db, &quest_id).await {
        Ok(rewards) => HttpResponse::Ok().json(QuestRewards { items: rewards }),
        Err(error) => HttpResponse::from_error(error),
    }
}

pub async fn get_quest_rewards_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    quest_id: &str,
) -> Result<Vec<QuestRewardItem>, QuestError> {
    match db.get_quest_reward_items(quest_id).await {
        Ok(rewards) => {
            if rewards.is_empty() {
                return Err(QuestError::QuestHasNoReward);
            }

            Ok(rewards)
        }
        Err(_) => {
            return Err(QuestError::CommonError(CommonError::Unknown));
        }
    }
}
