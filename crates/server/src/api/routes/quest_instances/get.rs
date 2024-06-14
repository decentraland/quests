use crate::{api::middlewares::RequiredAuthUser, domain::quests::QuestError};
use actix_web::{get, web, HttpResponse};
use quests_db::{
    core::definitions::{QuestInstance, QuestsDatabase},
    Database,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct GetQuestInstanceResponse {
    pub instance: QuestInstance,
}

/// Get a specific quest instance. Only the Quest Creator is allowed to see the Quest Instances
#[utoipa::path(
    params(
        ("quest_instance_id" = String, description = "Quest Instance UUID")
    ),
    responses(
        (status = 200, description = "Quest Instance", body = GetQuestInstanceResponse),
        (status = 401, description = "Unathorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/instances/{quest_instance_id}")]
pub async fn get_quest_instance(
    db: web::Data<Database>,
    quest_instance_id: web::Path<String>,
    auth_user: RequiredAuthUser,
) -> HttpResponse {
    let db = db.into_inner();
    let quest_instance_id = quest_instance_id.into_inner();

    let RequiredAuthUser { address } = auth_user;

    match db.get_quest_instance(&quest_instance_id).await {
        Ok(instance) => match db.is_quest_creator(&instance.quest_id, &address).await {
            Ok(is_creator) if !is_creator => HttpResponse::from_error(QuestError::NotQuestCreator),
            Ok(_) => HttpResponse::Ok().json(GetQuestInstanceResponse { instance }),
            Err(err) => {
                log::error!(
                    "error on checking if {address} is quest creator of {}",
                    instance.quest_id
                );
                HttpResponse::from_error(QuestError::from(err))
            }
        },
        Err(err) => {
            log::error!("error on getting quest instance {quest_instance_id} : {err}");
            HttpResponse::from_error(QuestError::from(err))
        }
    }
}
