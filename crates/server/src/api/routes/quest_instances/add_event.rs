use crate::{
    api::middlewares::RequiredAuthUser,
    domain::{events::add_event_controller, quests::QuestError},
};
use actix_web::{post, web, HttpResponse};
use quests_db::{core::definitions::QuestsDatabase, Database};
use quests_message_broker::messages_queue::RedisMessagesQueue;
use quests_protocol::definitions::EventRequest;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct AddEventToInstancePayload {
    pub event: EventRequest,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct AddEventToInstanceResponse {
    pub accepted: bool,
}

/// Get Quest Instance's state. Allowed for the Quest Creator
#[utoipa::path(
  params(
      ("quest_instance" = String, description = "Quest Instance UUID")
  ),
  responses(
      (status = 200, description = "Event enqueue result", body = AddEventToInstanceResponse),
      (status = 401, description = "Unauthorized"),
      (status = 403, description = "Forbidden"),
      (status = 404, description = "Quest Instance not found"),
      (status = 500, description = "Internal Server Error")
  )
)]
#[post("/instances/{quest_instance}/events")]
pub async fn add_event_to_instance(
    data: web::Data<Database>,
    events_queue: web::Data<RedisMessagesQueue>,
    quest_instance: web::Path<String>,
    event: web::Json<AddEventToInstancePayload>,
    auth_user: RequiredAuthUser,
) -> HttpResponse {
    let db = data.into_inner();

    let RequiredAuthUser { address } = auth_user;

    match db.get_quest_instance(&quest_instance).await {
        Ok(instance) => match db.is_quest_creator(&instance.quest_id, &address).await {
            Ok(is_creator) if !is_creator => HttpResponse::from_error(QuestError::NotQuestCreator),
            Ok(_) => {
                match add_event_controller(
                    events_queue.into_inner(),
                    &instance.user_address,
                    event.event.to_owned(),
                )
                .await
                {
                    Ok(_) => HttpResponse::Ok().json(AddEventToInstanceResponse { accepted: true }),
                    Err(_) => {
                        HttpResponse::Ok().json(AddEventToInstanceResponse { accepted: false })
                    }
                }
            }
            Err(err) => HttpResponse::from_error(QuestError::from(err)),
        },
        Err(err) => HttpResponse::from_error(QuestError::from(err)),
    }
}
