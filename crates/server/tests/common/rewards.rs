use std::collections::HashMap;

use quests_db::core::definitions::{QuestReward, QuestRewardHook, QuestRewardItem};

fn create_reward_hook() -> QuestRewardHook {
    let mut request_body = HashMap::new();
    request_body.insert("beneficiary".to_string(), "{user_address}".to_string());
    request_body.insert("campaign_key".to_string(), "eyJpZCI6ImJjMmQ1NWRjLWY3Y2Ut
NDEyOS05ODMxLWE5Nzk4ZTlmMTRiMSIsImNhbXBhaWduX2lkIjoiNjQ5YzVlMzgtYmVmOC00YmQ2LWIxM2YtYmQ2YTJiZGNjMDk2In0=.EC
ydl7nxWNUAgPWNgskHcFsqRGArULfHRtMyfc1UXIY=".to_string());
    QuestRewardHook {
        webhook_url: "https://rewards.decentraland.zone/api/campaigns/649c5e38-bef8-4bd6-b13f-bd6a2bdcc096/rewards".to_string(),
        request_body: Some(request_body),
    }
}

fn create_reward_item() -> QuestRewardItem {
    QuestRewardItem {
        name: "Macarena".to_string(),
        image_link: "https://peer.decentraland.zone/lambdas/collections/contents/urn:decentraland:matic:collections-v2:0xfb1d9d5dbb92f2dccc841bd3085081bb1bbeb04d:0/thumbnail".to_string(),
    }
}

pub fn create_reward() -> QuestReward {
    QuestReward {
        hook: create_reward_hook(),
        items: vec![create_reward_item()],
    }
}
