use quests_db::{
    core::{definitions::QuestsDatabase, errors::DBError},
    Database,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

pub async fn give_rewards_to_user(
    db: Arc<Database>,
    quest_id: &str,
    url: &str,
    user_address: &str,
) {
    match db.get_quest_reward(quest_id).await {
        Ok(quest_reward) => {
            match call_rewards_server(
                url,
                &quest_reward.campaign_id,
                &quest_reward.auth_key,
                user_address,
            )
            .await
            {
                Ok(assigned) => {
                    if assigned {
                        log::debug!("Processing event > Quest instance > Reward assigned");
                    } else {
                        log::debug!("Processing event > Quest instance > Reward was not assigned")
                    }
                }
                Err(error) => {
                    log::error!(
                        "Processing event > Quest instance > Failed to assign reward: {error:?}"
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
struct RewardsServerResponse {
    ok: bool,
}

async fn call_rewards_server(
    url: &str,
    campaign_id: &str,
    auth_key: &str,
    beneficary_address: &str,
) -> Result<bool, String> {
    let url = format!("https://{url}/api/campaigns/{campaign_id}/rewards");
    let client = reqwest::Client::new();
    let mut map = HashMap::new();
    map.insert("beneficiary", beneficary_address);
    map.insert("campaign_key", auth_key);
    if let Ok(response) = client.post(&url).json(&map).send().await {
        if let Ok(response) = response.json::<RewardsServerResponse>().await {
            if response.ok {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(String::from("Couldn't decode rewards server response"))
        }
    } else {
        Err(String::from("Couldn't call rewards server"))
    }
}
