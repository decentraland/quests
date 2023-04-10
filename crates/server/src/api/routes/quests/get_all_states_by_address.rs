use actix_web::{get, web, HttpResponse};
use quests_db::Database;
use quests_protocol::quests::{Quest, QuestState};
use serde::{Deserialize, Serialize};

use crate::domain::quests::get_all_quest_states_by_user_address_controller;

#[derive(Deserialize, Serialize)]
pub struct GetQuestStateByUserAddressResponse {
    pub states: Vec<(String, (Quest, QuestState))>,
}

#[utoipa::path(
    params(
        ("user_address" = String, description = "User's ethereum address")
    ),
    responses(
        (status = 200, description = "Quest States"),
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
