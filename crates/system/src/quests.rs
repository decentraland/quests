use std::sync::Arc;

use futures_util::future::join_all;
use quests_db::core::{definitions::QuestsDatabase, errors::DBError};
use quests_protocol::{
    definitions::{Event, ProtocolMessage, Quest, QuestDefinition, QuestState},
    quests::get_state,
};

#[derive(Debug)]
pub enum QuestStateCalculationError {
    DatabaseError(DBError),
    DefinitionError,
    StateError,
}

pub async fn get_quest(
    database: Arc<impl QuestsDatabase>,
    quest_id: &str,
) -> Result<Quest, QuestStateCalculationError> {
    let quest = database
        .get_quest(quest_id)
        .await
        .map_err(QuestStateCalculationError::DatabaseError)?;

    let quest_definition = QuestDefinition::decode(&*quest.definition)
        .map_err(|_| QuestStateCalculationError::DefinitionError)?;

    let quest = Quest {
        id: quest.id,
        name: quest.name,
        description: quest.description,
        creator_address: quest.creator_address,
        definition: Some(quest_definition),
        image_url: quest.image_url,
        active: quest.active,
        created_at: quest.created_at as u32,
    };
    Ok(quest)
}

pub async fn get_all_quest_states_by_user_address(
    database: Arc<impl QuestsDatabase + 'static>,
    user_address: &str,
) -> Result<Vec<(String, (Quest, QuestState))>, QuestStateCalculationError> {
    let quest_instances = database
        .get_active_user_quest_instances(user_address)
        .await
        .map_err(QuestStateCalculationError::DatabaseError)?;

    let mut join_handles = vec![];
    for quest_instance in quest_instances {
        let database = database.clone();
        let handle = tokio::spawn(async move {
            (
                quest_instance.id.clone(),
                get_instance_state(database, &quest_instance.quest_id, &quest_instance.id).await,
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
            Err(_) => return Err(QuestStateCalculationError::StateError),
        }
    }
    Ok(states)
}

pub async fn get_instance_state(
    database: Arc<impl QuestsDatabase>,
    quest_id: &str,
    quest_instance: &str,
) -> Result<(Quest, QuestState), QuestStateCalculationError> {
    let quest = get_quest(database.clone(), quest_id).await?;
    let stored_events = database
        .get_events(quest_instance)
        .await
        .map_err(QuestStateCalculationError::DatabaseError)?;

    let events = stored_events
        .iter()
        .map(|event| Event::decode(event.event.as_slice()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| QuestStateCalculationError::DefinitionError)?;

    let state = get_state(&quest, events);

    Ok((quest, state))
}
