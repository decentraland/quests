use crate::api::routes::{
    errors::CommonError,
    quests::{types::ToQuest, StartQuestRequest},
};
use futures_util::future::join_all;
use quests_db::core::definitions::{QuestInstance, QuestsDatabase};
use quests_protocol::{
    quest_state::get_state,
    quests::{Event, Quest, QuestState},
    ProtocolMessage,
};
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
}

pub async fn start_quest_controller(
    db: Arc<impl QuestsDatabase>,
    start_quest_request: StartQuestRequest,
) -> Result<String, QuestError> {
    db.get_quest(&start_quest_request.quest_id)
        .await
        .map_err(|err| -> QuestError { err.into() })?;

    db.start_quest(
        &start_quest_request.quest_id,
        &start_quest_request.user_address,
    )
    .await
    .map_err(|error| error.into())
}

pub async fn get_all_quest_states_by_user_address_controller(
    db: Arc<impl QuestsDatabase + 'static>,
    user_address: String,
) -> Result<Vec<(String, (Quest, QuestState))>, QuestError> {
    let quest_instances = db.get_user_quest_instances(&user_address).await?;
    let mut join_handles = vec![];
    for quest_instance in quest_instances {
        let db_cloned = db.clone();
        let handle = tokio::spawn(async move {
            (
                quest_instance.id.clone(),
                get_instance_state(db_cloned, quest_instance).await,
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
    quest_instance: QuestInstance,
) -> Result<(Quest, QuestState), QuestError> {
    let quest = db.get_quest(&quest_instance.quest_id).await;
    match quest {
        Ok(stored_quest) => {
            let quest = stored_quest.to_quest()?;
            let stored_events = db.get_events(&quest_instance.id).await?;

            let events = stored_events
                .iter()
                .map(|event| Event::decode(event.event.as_slice()))
                .collect::<Result<Vec<_>, _>>()?;

            let state = get_state(&quest, events);

            Ok((quest, state))
        }
        Err(_) => Err(QuestError::CommonError(CommonError::BadRequest(
            "the quest instance ID given doesn't correspond to a valid quest".to_string(),
        ))),
    }
}
