use super::definitions::{AddEvent, CreateQuest, QuestsDatabase};
use crate::core::{
    definitions::{QuestReward, QuestRewardHook, QuestRewardItem},
    errors::DBError,
};
use std::collections::HashMap;

pub async fn quest_database_works<DB: QuestsDatabase>(db: &DB, quest: CreateQuest<'_>) {
    assert!(db.ping().await);
    let quest_id = db.create_quest(&quest, "0xA").await.unwrap();

    let quest_reward = db.get_quest_reward_hook(&quest_id).await.unwrap_err();

    assert!(matches!(quest_reward, DBError::RowNotFound));

    db.add_reward_hook_to_quest(
        &quest_id,
        &QuestRewardHook {
            webhook_url: "https://rewards.webhook.com/{quest_id}".to_string(),
            request_body: Some(HashMap::from([(
                "beneficiary".to_owned(),
                "{user_address}".to_owned(),
            )])),
        },
    )
    .await
    .unwrap();

    db.add_reward_items_to_quest(
        &quest_id,
        &[QuestRewardItem {
            name: "SunGlasses".to_string(),
            image_link: "https://github.com/decentraland".to_string(),
        }],
    )
    .await
    .unwrap();

    let quest_reward = db.get_quest_reward_hook(&quest_id).await.unwrap();

    assert_eq!(
        quest_reward.webhook_url,
        "https://rewards.webhook.com/{quest_id}"
    );

    assert_eq!(
        quest_reward.request_body,
        Some(HashMap::from([(
            "beneficiary".to_owned(),
            "{user_address}".to_owned()
        )]))
    );

    let quest_reward_items = db.get_quest_reward_items(&quest_id).await.unwrap();

    assert_eq!(quest_reward_items.len(), 1);
    assert_eq!(quest_reward_items[0].name, "SunGlasses");
    assert_eq!(
        quest_reward_items[0].image_link,
        "https://github.com/decentraland"
    );

    let is_active = db.is_active_quest(&quest_id).await.unwrap();
    assert!(is_active);

    let updated_quest = CreateQuest {
        name: "UPDATED_QUEST",
        description: quest.description,
        definition: quest.definition.clone(),
        image_url: quest.image_url,
        reward: None,
    };

    // updatable checks
    assert!(db.is_updatable(&quest_id).await.unwrap());
    let new_quest_id = db
        .update_quest(&quest_id, &updated_quest, "0xA")
        .await
        .unwrap();
    let is_active = db.is_active_quest(&quest_id).await.unwrap();
    assert!(!is_active);
    assert!(!db.is_updatable(&quest_id).await.unwrap());
    assert!(!db.can_activate_quest(&quest_id).await.unwrap());

    // creators quests should be ONE because query returns current versions (activated and deactivated) and not old versions
    let quests_by_creator = db.get_quests_by_creator_id("0xA", 0, 50).await.unwrap();
    assert_eq!(quests_by_creator.len(), 1);
    assert_eq!(quests_by_creator.get(0).unwrap().id, new_quest_id);
    assert!(quests_by_creator.get(0).unwrap().active);
    let create_deactivated_quest = CreateQuest {
        name: "DEACTIVATED_CREATOR_QUEST",
        description: quest.description,
        definition: quest.definition.clone(),
        image_url: quest.image_url,
        reward: None,
    };
    let deactivated_quest = db
        .create_quest(&create_deactivated_quest, "0xA")
        .await
        .unwrap();
    db.deactivate_quest(&deactivated_quest).await.unwrap();
    // creators quests should be TWO because query returns current versions (activated and deactivated) and not old versions
    let quests_by_creator = db.get_quests_by_creator_id("0xA", 0, 50).await.unwrap();
    println!("quests {quests_by_creator:?}");
    assert_eq!(quests_by_creator.len(), 2);
    // order by desc
    assert_eq!(quests_by_creator.get(0).unwrap().id, deactivated_quest);
    assert!(!quests_by_creator.get(0).unwrap().active);
    assert_eq!(quests_by_creator.get(1).unwrap().id, new_quest_id);
    assert!(quests_by_creator.get(1).unwrap().active);

    // new quest old versions
    let old_versions = db.get_old_quest_versions(&new_quest_id).await.unwrap();
    assert_eq!(old_versions.len(), 1);
    assert_eq!(old_versions.get(0).unwrap(), &quest_id);

    // old quest is still there
    let get_old_quest = db.get_quest(&quest_id).await.unwrap();
    assert_eq!(get_old_quest.id, quest_id);
    assert_eq!(get_old_quest.description, quest.description);
    assert_eq!(get_old_quest.definition, quest.definition);

    // old quest should not be active
    let active_quests = db.get_active_quests(0, 2).await.unwrap();
    assert_eq!(active_quests.len(), 1);
    assert!(active_quests.iter().any(|quest| quest.id == new_quest_id));
    assert!(!active_quests.iter().any(|quest| quest.id == quest_id));

    let quest_instance_id = db.start_quest(&quest_id, "0xA").await.unwrap();

    let get_quest_instance = db.get_quest_instance(&quest_instance_id).await.unwrap();

    assert_eq!(get_quest_instance.user_address, "0xA");
    assert_eq!(get_quest_instance.quest_id, quest_id);

    let instances_by_quest_id = db.get_quest_instances_by_quest_id(&quest_id).await.unwrap();

    assert_eq!(instances_by_quest_id.0.len(), 1);
    assert_eq!(instances_by_quest_id.1.len(), 0);

    let event = AddEvent {
        id: uuid::Uuid::new_v4().to_string(),
        user_address: "0xA",
        event: vec![0],
    };

    db.add_event(&event, &quest_instance_id).await.unwrap();

    let quest_instance_events = db.get_events(&quest_instance_id).await.unwrap();

    assert_eq!(quest_instance_events.len(), 1);
    assert_eq!(
        quest_instance_events[0].quest_instance_id,
        quest_instance_id
    );
    assert_eq!(quest_instance_events[0].user_address, "0xA");
    assert_eq!(quest_instance_events[0].event, vec![0]);

    let is_active_instance = db
        .is_active_quest_instance(&get_quest_instance.id)
        .await
        .unwrap();
    assert!(is_active_instance);

    let active_instances = db.get_active_user_quest_instances("0xA").await.unwrap();
    assert_eq!(active_instances.len(), 1);

    db.abandon_quest_instance(&get_quest_instance.id)
        .await
        .unwrap();
    let is_active_instance = db
        .is_active_quest_instance(&get_quest_instance.id)
        .await
        .unwrap();
    assert!(!is_active_instance);

    let new_instance = db.start_quest(&quest_id, "0xB").await.unwrap();
    db.complete_quest_instance(&new_instance).await.unwrap();

    let result = db.is_completed_instance(&new_instance).await.unwrap();
    assert!(result);

    let active_quests = db.get_active_quests(0, 10).await.unwrap();

    assert_eq!(active_quests.len(), 1);
    assert_eq!(active_quests[0].name, "UPDATED_QUEST");

    db.deactivate_quest(&new_quest_id).await.unwrap();

    assert!(db.can_activate_quest(&new_quest_id).await.unwrap());
    assert!(db.is_updatable(&new_quest_id).await.unwrap());

    let active_quests = db.get_active_quests(0, 10).await.unwrap();
    assert_eq!(active_quests.len(), 0);

    assert!(db.activate_quest(&new_quest_id).await.unwrap());

    let active_quests = db.get_active_quests(0, 10).await.unwrap();
    assert_eq!(active_quests.len(), 1);

    // create quest with TX
    let items = vec![QuestRewardItem {
        name: "SunGlasses".to_string(),
        image_link: "https://github.com/decentraland".to_string(),
    }];
    let hook = QuestRewardHook {
        webhook_url: "https://rewards.webhook.com/{quest_id}".to_string(),
        request_body: Some(HashMap::from([(
            "beneficiary".to_owned(),
            "{user_address}".to_owned(),
        )])),
    };
    let quest_w_reward_id = db
        .create_quest(
            &CreateQuest {
                reward: Some(QuestReward {
                    hook: hook.clone(),
                    items: items.clone(),
                }),
                ..quest
            },
            "0xB",
        )
        .await
        .unwrap();
    let quest_w_reward = db.get_quest(&quest_w_reward_id).await.unwrap();
    assert_eq!(quest.name, quest_w_reward.name);
    assert_eq!(quest.description, quest_w_reward.description);
    assert_eq!(quest.image_url, quest_w_reward.image_url);

    let quest_w_reward_hook = db.get_quest_reward_hook(&quest_w_reward_id).await.unwrap();
    assert_eq!(quest_w_reward_hook.webhook_url, hook.webhook_url);

    let quest_w_reward_items = db.get_quest_reward_items(&quest_w_reward_id).await.unwrap();
    assert_eq!(quest_reward_items.len(), 1);
    assert_eq!(quest_w_reward_items.get(0).unwrap().name, "SunGlasses");
    assert_eq!(
        quest_w_reward_items.get(0).unwrap().image_link,
        "https://github.com/decentraland"
    )
}
