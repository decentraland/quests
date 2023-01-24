use super::{
    definitions::{AddEvent, CreateQuest, QuestsDatabase, UpdateQuest},
    errors::DBError,
};

pub async fn quest_database_works<DB: QuestsDatabase>(db: &DB, quest: CreateQuest<'_>) {
    assert!(db.ping().await);
    let quest_id = db.create_quest(&quest).await.unwrap();

    let update_quest = UpdateQuest {
        name: "UPDATED_QUEST",
        description: quest.description,
        definition: quest.definition.clone(),
    };

    db.update_quest(&quest_id, &update_quest).await.unwrap();

    let get_quest = db.get_quest(&quest_id).await.unwrap();

    assert_eq!(get_quest.id, quest_id);
    assert_eq!(get_quest.description, quest.description);
    assert_eq!(get_quest.definition, quest.definition);

    let quest_instance_id = db.start_quest(&quest_id, "0xA").await.unwrap();

    let get_quest_instance = db.get_quest_instance(&quest_instance_id).await.unwrap();

    assert_eq!(get_quest_instance.user_address, "0xA");
    assert_eq!(get_quest_instance.quest_id, quest_id);

    let event = AddEvent {
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

    let get_all_quests = db.get_quests(0, 10).await.unwrap();

    assert_eq!(get_all_quests.len(), 1);
    assert_eq!(get_all_quests[0].name, "UPDATED_QUEST");

    db.delete_quest(&quest_id).await.unwrap();

    let err = db.get_quest(&quest_id).await.unwrap_err();
    assert!(matches!(err, DBError::RowNotFound))
}
