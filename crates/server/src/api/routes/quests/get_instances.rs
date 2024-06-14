use crate::{api::middlewares::RequiredAuthUser, domain::quests::QuestError};
use actix_web::{get, web, HttpResponse};
use quests_db::{
    core::definitions::{QuestInstance, QuestsDatabase},
    Database,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct GetQuestInstancesQuery {
    offset: Option<i64>,
    limit: Option<i64>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct GetQuestInstancesResponse {
    pub instances: Vec<QuestInstance>,
    pub total: i64,
}

/// Get all quest instances. Only the Quest Creator is allowed to see the Quest Instances
#[utoipa::path(
    params(
        ("query" = GetQuestsQuery, Query, description = "Offset and limit params"),
        ("quest_id" = String, description = "Quest UUID")
    ),
    responses(
        (status = 200, description = "Quest's Instances", body = GetQuestInstacesResponse),
        (status = 401, description = "Unathorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/quests/{quest_id}/instances")]
pub async fn get_quest_instances(
    db: web::Data<Database>,
    quest_id: web::Path<String>,
    query: web::Query<GetQuestInstancesQuery>,
    auth_user: RequiredAuthUser,
) -> HttpResponse {
    let db = db.into_inner();
    let quest_id = quest_id.into_inner();

    let RequiredAuthUser { address } = auth_user;

    match db.is_quest_creator(&quest_id, &address).await {
        Ok(is_creator) if !is_creator => HttpResponse::from_error(QuestError::NotQuestCreator),
        Ok(_) => match db
            .get_active_quest_instances_by_quest_id(
                &quest_id,
                query.offset.unwrap_or(0),
                query.limit.unwrap_or(50),
            )
            .await
        {
            Ok(instances) => match db.count_active_quest_instances_by_quest_id(&quest_id).await {
                Ok(total) => {
                    HttpResponse::Ok().json(GetQuestInstancesResponse { instances, total })
                }
                Err(err) => {
                    log::error!("error on counting quest instances {err} for {quest_id}");
                    HttpResponse::from_error(QuestError::from(err))
                }
            },
            Err(err) => {
                log::error!("error on getting quest instances {err} for {quest_id}");
                HttpResponse::from_error(QuestError::from(err))
            }
        },
        Err(err) => {
            log::error!("error on checking quest creator");
            HttpResponse::from_error(QuestError::from(err))
        }
    }
}
