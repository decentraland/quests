use crate::{api::middlewares::RequiredAuthUser, domain::quests::QuestError};
use actix_web::{patch, web, HttpResponse};
use quests_db::{core::definitions::QuestsDatabase, Database};
use std::sync::Arc;

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

    match reset_quest_instance_controller(db, &address, &quest_instance).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn reset_quest_instance_controller(
    db: Arc<impl QuestsDatabase>,
    auth_user_address: &str,
    quest_instance_id: &str,
) -> Result<(), QuestError> {
    match db.get_quest_instance(quest_instance_id).await {
        Ok(instance) => match db.get_quest(&instance.quest_id).await {
            Ok(quest) => {
                if !auth_user_address.eq_ignore_ascii_case(&quest.creator_address) {
                    return Err(QuestError::ResetQuestInstanceNotAllowed);
                }

                // remove events to reset quest instance state
                db.remove_events(quest_instance_id).await.map_err(|err| {
                    let err: QuestError = err.into();
                    err
                })?;

                db.remove_instance_from_completed_instances(quest_instance_id)
                    .await
                    .map_err(|err| {
                        let err: QuestError = err.into();
                        err
                    })?;

                Ok(())
            }
            Err(err) => {
                log::error!("Error getting quest: {:?}", err);
                let err: QuestError = err.into();
                Err(err)
            }
        },
        Err(err) => {
            log::error!("Error getting quest instance: {:?}", err);
            let err: QuestError = err.into();
            Err(err)
        }
    }
}
