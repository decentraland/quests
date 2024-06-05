use crate::api::routes::errors::CommonError;
use quests_db::core::definitions::QuestsDatabase;
use quests_system::{get_instance_state, QuestStateCalculationError};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuestError {
    #[error("{0}")]
    CommonError(CommonError),
    #[error("Quest Definition issue")]
    DeserializationError,
    #[error("Quest Validation Error: {0}")]
    QuestValidation(String),
    #[error("Cannot modify a quest if you are not the quest creator")]
    NotQuestCreator,
    #[error("Cannot modify a quest instance if you are not the user playing the quest")]
    NotInstanceOwner,
    #[error("Quest doesn't exist or is inactive")]
    NotFoundOrInactive,
    #[error("Quest already started and active")]
    QuestAlreadyStarted,
    #[error("Quest already completed")]
    QuestAlreadyCompleted,
    #[error("Quest has no reward")]
    QuestHasNoReward,
    #[error("Requested Quest cannot be activated because it may be prevoiusly updated and replaced with a new Quest or it may be already active")]
    QuestNotActivable,
    #[error("Requested Quest was previously updated and replaced with a new Quest")]
    QuestIsNotUpdatable,
    #[error("Quest is currently deactivated")]
    QuestIsCurrentlyDeactivated,
    #[error("Cannot reset a Quest Instance if you are not the Quest Creator")]
    ResetQuestInstanceNotAllowed,
}

pub async fn abandon_quest(
    db: Arc<impl QuestsDatabase>,
    user_address: &str,
    quest_instance_id: &str,
) -> Result<(), QuestError> {
    let quest_instance = db.get_quest_instance(quest_instance_id).await?;
    if quest_instance.user_address != user_address {
        return Err(QuestError::NotInstanceOwner);
    }

    let (_, quest_state) =
        get_instance_state(db.clone(), &quest_instance.quest_id, &quest_instance.id).await?;
    if quest_state.is_completed() {
        return Err(QuestError::QuestAlreadyCompleted);
    }

    _ = db.abandon_quest_instance(quest_instance_id).await?;
    Ok(())
}

pub async fn start_quest(
    db: Arc<impl QuestsDatabase>,
    user_address: &str,
    quest_id: &str,
) -> Result<String, QuestError> {
    if !db.is_active_quest(quest_id).await? {
        return Err(QuestError::NotFoundOrInactive);
    }

    if db.has_active_quest_instance(user_address, quest_id).await? {
        return Err(QuestError::QuestAlreadyStarted);
    }

    Ok(db.start_quest(quest_id, user_address).await?)
}

pub async fn reset_quest_instance(
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

impl From<QuestStateCalculationError> for QuestError {
    fn from(value: QuestStateCalculationError) -> Self {
        match value {
            QuestStateCalculationError::DatabaseError(e) => QuestError::CommonError(e.into()),
            QuestStateCalculationError::DefinitionError => QuestError::DeserializationError,
            QuestStateCalculationError::StateError => {
                QuestError::QuestValidation("Quest state error".to_string())
            }
        }
    }
}
