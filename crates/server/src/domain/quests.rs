use crate::api::routes::errors::CommonError;
use futures_util::future::join_all;
use quests_db::core::definitions::QuestsDatabase;
use quests_protocol::definitions::*;
use quests_protocol::quests::get_state;
use std::sync::Arc;
use thiserror::Error;

use super::types::ToQuest;

#[derive(Error, Debug)]
pub enum QuestError {
    #[error("{0}")]
    CommonError(CommonError),
    #[error("Quest Definition issue")]
    DeserializationError,
    #[error("Quest Validation Error: {0}")]
    QuestValidation(String),
    #[error("Cannot modify a quest if you are not the user playing the quest")]
    NotInstanceOwner,
    #[error("Quest doesn't exist or is inactive")]
    NotFoundOrInactive,
    #[error("Quest already started and active")]
    QuestAlreadyStarted,
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

    _ = db.abandon_quest(quest_instance_id).await?;
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

    let active_quests = db.get_active_user_quest_instances(user_address).await?;
    for quest in active_quests {
        if quest.quest_id == quest_id {
            return Err(QuestError::QuestAlreadyStarted);
        }
    }

    Ok(db.start_quest(quest_id, user_address).await?)
}

pub async fn get_all_quest_states_by_user_address(
    db: Arc<impl QuestsDatabase + 'static>,
    user_address: String,
) -> Result<Vec<(String, (Quest, QuestState))>, QuestError> {
    let quest_instances = db.get_active_user_quest_instances(&user_address).await?;

    let mut join_handles = vec![];
    for quest_instance in quest_instances {
        let db_cloned = db.clone();
        let handle = tokio::spawn(async move {
            (
                quest_instance.id.clone(),
                get_instance_state(db_cloned, &quest_instance.quest_id, &quest_instance.id).await,
            )
        });
        join_handles.push(handle);
    }
    let join_results = join_all(join_handles).await;
    let mut states = vec![];
    for join_result in join_results {
        match join_result {
            Ok((id, state_result)) => match state_result {
                Ok(state) => states.push((id, state)),
                Err(quest_error) => return Err(quest_error),
            },
            Err(_) => return Err(QuestError::CommonError(CommonError::Unknown)),
        }
    }
    Ok(states)
}

pub async fn get_instance_state(
    db: Arc<impl QuestsDatabase>,
    quest_id: &str,
    quest_instance: &str,
) -> Result<(Quest, QuestState), QuestError> {
    let quest = db.get_quest(quest_id).await.map_err(|_| {
        QuestError::CommonError(CommonError::BadRequest(
            "the quest instance ID given doesn't correspond to a valid quest".to_string(),
        ))
    })?;
    let stored_events = db.get_events(quest_instance).await?;

    let events = stored_events
        .iter()
        .map(|event| Event::decode(event.event.as_slice()))
        .collect::<Result<Vec<_>, _>>()?;

    let quest = quest.to_quest()?;
    let state = get_state(&quest, events);

    Ok((quest, state))
}

pub async fn get_quest<DB: QuestsDatabase>(db: Arc<DB>, id: String) -> Result<Quest, QuestError> {
    let stored_quest = db.get_quest(&id).await?;
    stored_quest.to_quest()
}
