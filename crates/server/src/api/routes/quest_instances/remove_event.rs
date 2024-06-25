use crate::{api::middlewares::RequiredAuthUser, domain::quests::QuestError};
use actix_web::{delete, web, HttpResponse};
use quests_db::{core::definitions::QuestsDatabase, Database};

/// Get Quest Instance's state. Allowed for the Quest Creator
#[utoipa::path(
params(
    ("quest_instance" = String, description = "Quest Instance UUID")
),
responses(
    (status = 204, description = "Event removed"),
    (status = 401, description = "Unauthorized"),
    (status = 403, description = "Forbidden"),
    (status = 404, description = "Quest Instance or Event not found"),
    (status = 500, description = "Internal Server Error")
)
)]
#[delete("/instances/{quest_instance}/events/{event_id}")]
pub async fn remove_event_from_instance(
    data: web::Data<Database>,
    path: web::Path<(String, String)>,
    auth_user: RequiredAuthUser,
) -> HttpResponse {
    let db = data.into_inner();

    let (quest_instance_id, event_id) = path.into_inner();

    let RequiredAuthUser { address } = auth_user;

    match db.get_quest_instance(&quest_instance_id).await {
        Ok(instance) => match db.is_quest_creator(&instance.quest_id, &address).await {
            Ok(is_creator) if !is_creator => HttpResponse::from_error(QuestError::NotQuestCreator),
            Ok(_) => match db.remove_event(&event_id).await {
                // we remove it from completed instance in case it's already completed
                Ok(_) => match db
                    .remove_instance_from_completed_instances(&instance.id)
                    .await
                {
                    Ok(_) => HttpResponse::NoContent().finish(),
                    Err(err) => HttpResponse::from_error(QuestError::from(err)),
                },
                Err(err) => HttpResponse::from_error(QuestError::from(err)),
            },
            Err(err) => HttpResponse::from_error(QuestError::from(err)),
        },
        Err(err) => HttpResponse::from_error(QuestError::from(err)),
    }
}
