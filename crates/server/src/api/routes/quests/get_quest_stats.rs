use actix_web::{get, web, HttpResponse};
use futures_util::{stream::FuturesUnordered, StreamExt};
use quests_db::{core::definitions::QuestsDatabase, Database};
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use utoipa::ToSchema;

use crate::{api::middlewares::RequiredAuthUser, domain::quests::QuestError};

#[derive(Deserialize, Serialize, ToSchema)]
pub struct GetQuestStatsResponse {
    pub active_players: usize,
    pub abandoned: usize,
    pub completed: usize,
    pub started_in_last_24_hours: usize,
}

/// Get a quest stats
#[utoipa::path(
    params(
        ("quest_id" = String, description = "Quest ID")
    ),
    responses(
        (status = 200, description = "Quest Stats", body = GetQuestStatsResponse),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unathorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Quest not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/quests/{quest_id}/stats")]
pub async fn get_quest_stats(
    db: web::Data<Database>,
    quest_id: web::Path<String>,
    auth_user: RequiredAuthUser,
) -> HttpResponse {
    let db = db.into_inner();
    let quest_id = quest_id.into_inner();

    let RequiredAuthUser { address } = auth_user;

    match get_quest_stats_controller(db, &quest_id, &address).await {
        Ok(quest_stats) => HttpResponse::Ok().json(quest_stats),
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn get_quest_stats_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    quest_id: &str,
    user_address: &str,
) -> Result<GetQuestStatsResponse, QuestError> {
    let mut futs = FuturesUnordered::new();

    match db.is_quest_creator(quest_id, user_address).await {
        Ok(is_creator) if !is_creator => Err(QuestError::NotQuestCreator),
        Ok(_) => match db.get_all_quest_instances_by_quest_id(quest_id).await {
            Ok((actives, abandoned)) => {
                let mut stats = GetQuestStatsResponse {
                    active_players: actives.len(),
                    abandoned: abandoned.len(),
                    completed: 0,
                    started_in_last_24_hours: 0,
                };

                // TODO: All computation should be replaced by a cronjob and not done on demand
                for active in &actives {
                    let db_clone = db.clone();
                    let instance_id: String = active.id.clone();
                    if is_within_24_hours(active.start_timestamp) {
                        stats.started_in_last_24_hours += 1;
                    }
                    futs.push(async move { db_clone.is_completed_instance(&instance_id).await });
                }

                while let Some(Ok(is_completed)) = futs.next().await {
                    if is_completed {
                        stats.completed += 1;
                    }
                }

                Ok(stats)
            }
            Err(err) => {
                log::error!(
                    "> get_quest_stats_controller > Failed to get quest stats: {}",
                    err
                );
                Err(QuestError::from(err))
            }
        },
        Err(err) => Err(err.into()),
    }
}

fn is_within_24_hours(timestamp: i64) -> bool {
    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get current timestamp")
        .as_secs() as i64;

    let twenty_four_hours = 24 * 60 * 60; // 24 hours in seconds

    current_timestamp - timestamp <= twenty_four_hours
}
