use crate::{
    api::{middlewares::OptionalAuthUser, routes::errors::CommonError},
    domain::quests::QuestError,
};
use actix_web::{get, web, HttpResponse};
use quests_db::{
    core::{
        definitions::{QuestRewardHook, QuestRewardItem, QuestsDatabase},
        errors::DBError,
    },
    Database,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::join;
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct GetQuestRewardResponse {
    pub items: Vec<QuestRewardItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook: Option<QuestRewardHook>,
}

#[derive(Deserialize, IntoParams)]
pub struct GetQuestRewardsParams {
    with_hook: Option<bool>,
}

/// Get a quest rewards
/// Returns the quest rewards
#[utoipa::path(
    params(
        ("quest_id" = String, description = "Quest UUID")
    ),
    responses(
        (status = 200, description = "Quest Rewards", body = GetQuestRewardResponse),
        (status = 404, description = "Not found rewards"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/quests/{quest_id}/reward")]
pub async fn get_quest_reward(
    quest_id: web::Path<String>,
    db: web::Data<Database>,
    query_params: web::Query<GetQuestRewardsParams>,
    auth_user: OptionalAuthUser,
) -> HttpResponse {
    let quest_id = quest_id.into_inner();
    let db = db.into_inner();

    match get_quest_rewards_controller(
        auth_user.address,
        db,
        &quest_id,
        query_params.with_hook.unwrap_or(false),
    )
    .await
    {
        Ok(rewards) => match rewards {
            Rewards::Items(items) => {
                HttpResponse::Ok().json(GetQuestRewardResponse { items, hook: None })
            }
            Rewards::WithHook { items, hook } => HttpResponse::Ok().json(GetQuestRewardResponse {
                items,
                hook: Some(hook),
            }),
        },
        Err(error) => HttpResponse::from_error(error),
    }
}

enum Rewards {
    Items(Vec<QuestRewardItem>),
    WithHook {
        items: Vec<QuestRewardItem>,
        hook: QuestRewardHook,
    },
}

async fn get_quest_rewards_controller<DB: QuestsDatabase>(
    user: Option<String>,
    db: Arc<DB>,
    quest_id: &str,
    mut with_hook: bool,
) -> Result<Rewards, QuestError> {
    if let Some(user_address) = user {
        let is_creator = db
            .is_quest_creator(quest_id, &user_address)
            .await
            .map_err(QuestError::from)?;

        if !is_creator {
            with_hook = false
        }
    } else {
        with_hook = false
    }

    if with_hook {
        let futures = join!(
            db.get_quest_reward_items(quest_id),
            db.get_quest_reward_hook(quest_id)
        );
        match (futures.0, futures.1) {
            (Ok(rewards), Ok(hook)) => {
                if rewards.is_empty() {
                    return Err(QuestError::QuestHasNoReward);
                }

                Ok(Rewards::WithHook {
                    items: rewards,
                    hook,
                })
            }
            (Err(err), _) | (_, Err(err)) => {
                if matches!(err, DBError::RowNotFound) {
                    return Err(QuestError::CommonError(CommonError::NotFound));
                }
                Err(QuestError::CommonError(CommonError::Unknown))
            }
        }
    } else {
        match db.get_quest_reward_items(quest_id).await {
            Ok(rewards) if rewards.is_empty() => Err(QuestError::QuestHasNoReward),
            Ok(rewards) => Ok(Rewards::Items(rewards)),
            Err(_) => Err(QuestError::CommonError(CommonError::Unknown)),
        }
    }
}
