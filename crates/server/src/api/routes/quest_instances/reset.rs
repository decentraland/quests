use crate::{api::middlewares::RequiredAuthUser, domain::quests};
use actix_web::{patch, web, HttpResponse};
use quests_db::Database;
use quests_protocol::definitions::Quest;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GetCreatorQuestsResponse {
    pub quests: Vec<Quest>,
}

/// Reset a User's Quest Instance. It can only be executed by the Quest Creator
#[utoipa::path(
  params(
      ("quest_instance" = String, description = "Quest Instance UUID")
  ),
  responses(
      (status = 204, description = "Quest Instance was reset"),
      (status = 401, description = "Unauthorized"),
      (status = 403, description = "Cannot reset a Quest Instance if you are not the Quest Creator"),
      (status = 404, description = "Quest Instance not found"),
      (status = 500, description = "Internal Server Error")
  )
)]
#[patch("/instances/{quest_instance}/reset")]
pub async fn reset_quest_instance(
    data: web::Data<Database>,
    quest_instance: web::Path<String>,
    auth_user: RequiredAuthUser,
) -> HttpResponse {
    let db = data.into_inner();

    let RequiredAuthUser { address } = auth_user;

    match quests::reset_quest_instance(db, &address, &quest_instance).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(err) => HttpResponse::from_error(err),
    }
}
