use crate::{api::routes::quests::get_user_address_from_request, domain::quests};
use actix_web::{patch, web, HttpRequest, HttpResponse};
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
    req: HttpRequest,
    data: web::Data<Database>,
    quest_instance: web::Path<String>,
) -> HttpResponse {
    let db = data.into_inner();

    let auth_user = get_user_address_from_request(&req).unwrap(); // unwrap here is safe

    match quests::reset_quest_instance(db, &auth_user, &quest_instance).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(err) => HttpResponse::from_error(err),
    }
}
