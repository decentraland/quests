use quests_db::{
    core::{definitions::QuestsDatabase, errors::DBError},
    Database,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

pub async fn give_rewards_to_user(db: Arc<Database>, quest_id: &str, user_address: &str) {
    match db.get_quest_reward_hook(quest_id).await {
        Ok(quest_reward) => {
            match call_rewards_hook(
                &quest_reward.webhook_url,
                quest_reward.request_body,
                quest_id,
                user_address,
            )
            .await
            {
                Ok(assigned) => {
                    if assigned {
                        log::info!("Processing event > Quest instance > Reward assigned > Quest ID: {quest_id} / User: {user_address}");
                    } else {
                        log::info!("Processing event > Quest instance > Reward assigned > Quest ID: {quest_id} / User: {user_address}");
                    }
                }
                Err(error) => {
                    log::error!(
                        "Processing event > Quest instance > Failed to assign reward: {error:?} > Quest ID: {quest_id} / User: {user_address}"
                    );
                }
            }
        }
        Err(err) => match err {
            DBError::RowNotFound => {
                log::debug!("Processing event > Quest instance > Quest has no reward");
            }
            DBError::GetQuestRewardFailed(err) => {
                log::error!(
                    "Processing event > Quest instance > Failed to get quest reward: {err:?}"
                );
            }
            _ => {}
        },
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RewardsHookResponse {
    ok: bool,
}

async fn call_rewards_hook(
    url: &str,
    body: Option<HashMap<String, String>>,
    quest_id: &str,
    user_address: &str,
) -> Result<bool, String> {
    let url_parsed = quests_protocol::rewards::rewards_parser(url, quest_id, user_address);
    let mut client = reqwest::Client::new().post(&url_parsed);

    if let Some(mut body) = body {
        for (_, v) in body.iter_mut() {
            *v = quests_protocol::rewards::rewards_parser(v, quest_id, user_address);
        }
        client = client.json(&body);
    }

    if let Ok(response) = client.send().await {
        if let Ok(response) = response.json::<RewardsHookResponse>().await {
            if response.ok {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(String::from("Couldn't decode rewards hook response"))
        }
    } else {
        Err(String::from("Couldn't call rewards hook"))
    }
}
