use std::sync::Arc;

use actix_web::{get, web, HttpResponse};
use futures_util::future::join_all;
use quests_db::{core::definitions::QuestsDatabase, Database};
use quests_definitions::quest_state::QuestState;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::routes::errors::{CommonError, QuestError};

use super::get_instance_state;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct GetQuestStateByUserAddressResponse {
    pub states: Vec<QuestState>,
}

#[utoipa::path(
    params(
        ("user_address" = String, description = "User's ethereum address")
    ),
    responses(
        (status = 200, description = "Quest States", body = [GetQuestStateByUserAddressResponse]),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Quest not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/quests/instances/{user_address}")]
pub async fn get_all_quest_states_by_user_address(
    data: web::Data<Database>,
    user_address: web::Path<String>,
) -> HttpResponse {
    let db = data.into_inner();
    match get_all_quest_states_by_user_address_controller(db, user_address.into_inner()).await {
        Ok(quest_state) => HttpResponse::Ok().json(GetQuestStateByUserAddressResponse {
            states: quest_state,
        }),
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn get_all_quest_states_by_user_address_controller<DB: QuestsDatabase + 'static>(
    db: Arc<DB>,
    user_address: String,
) -> Result<Vec<QuestState>, QuestError> {
    let quest_instances = db.get_user_quest_instances(&user_address).await?;
    let mut join_handles = vec![];
    for quest_instance in quest_instances {
        let db_cloned = db.clone();
        let handle =
            actix_web::rt::spawn(
                async move { get_instance_state(db_cloned, quest_instance).await },
            );
        join_handles.push(handle);
    }
    let join_results = join_all(join_handles).await;
    let mut states = vec![];
    for join_result in join_results {
        match join_result {
            Ok(state_result) => match state_result {
                Ok(state) => states.push(state),
                Err(quest_error) => return Err(quest_error),
            },
            Err(_) => return Err(QuestError::CommonError(CommonError::Unknown)),
        }
    }
    Ok(states)
}
