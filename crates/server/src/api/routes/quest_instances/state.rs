use crate::{api::middlewares::RequiredAuthUser, domain::quests::QuestError};
use actix_web::{get, web, HttpResponse};
use quests_db::{
    core::definitions::{Event, QuestsDatabase},
    Database,
};
use quests_protocol::definitions::QuestState;
use quests_system::get_instance_state;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GetInstanceStateResponse {
    pub state: QuestState,
    pub events: Vec<Event>,
}

/// Get Quest Instance's state. Allowed for the Quest Creator
#[utoipa::path(
  params(
      ("quest_instance" = String, description = "Quest Instance UUID")
  ),
  responses(
      (status = 200, description = "Quest Instance state", body = GetInstanceStateResponse),
      (status = 401, description = "Unauthorized"),
      (status = 403, description = "Forbidden"),
      (status = 404, description = "Quest Instance not found"),
      (status = 500, description = "Internal Server Error")
  )
)]
#[get("/instances/{quest_instance}/state")]
pub async fn get_quest_instance_state(
    data: web::Data<Database>,
    quest_instance: web::Path<String>,
    auth_user: RequiredAuthUser,
) -> HttpResponse {
    let db = data.into_inner();

    let RequiredAuthUser { address } = auth_user;

    match db.get_quest_instance(&quest_instance).await {
        Ok(instance) => match db.is_quest_creator(&instance.quest_id, &address).await {
            Ok(is_creator) if !is_creator => HttpResponse::from_error(QuestError::NotQuestCreator),
            Ok(_) => match get_instance_state(db.clone(), &instance.quest_id, &instance.id).await {
                Ok((_, state, events)) => {
                    HttpResponse::Ok().json(GetInstanceStateResponse { state, events })
                }
                Err(err) => HttpResponse::from_error(QuestError::from(err)),
            },
            Err(err) => HttpResponse::from_error(QuestError::from(err)),
        },
        Err(err) => HttpResponse::from_error(QuestError::from(err)),
    }
}
