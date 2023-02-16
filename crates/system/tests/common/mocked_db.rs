
#[derive(Clone)]
struct MockedDatabase {
    quests: HashMap<String, StoredQuest>,
    quest_instances: HashMap<String, QuestInstance>,
    events: HashMap<String, Vec<Event>>,
}

#[async_trait]
impl QuestsDatabase for MockedDatabase {
    async fn ping(&self) -> bool {
        true
    }

    async fn get_quests(&self, offset: i64, limit: i64) -> DBResult<Vec<StoredQuest>> {
        let quests = self.quests.iter().map(|(k, v)| v.clone()).collect();
        Ok(quests)
    }

    async fn create_quest(&self, quest: &CreateQuest) -> DBResult<String> {
        let id = self.quests.len().to_string();
        let stored_quest = StoredQuest {
            id,
            name: quest.name.to_string(),
            description: quest.description.to_string(),
            definition: quest.definition,
        };

        self.quests.insert(id, stored_quest);
        
        Ok(id)
    }
    async fn update_quest(&self, quest_id: &str, quest: &UpdateQuest) -> DBResult<()> {
        Ok(())
    }
    async fn get_quest(&self, id: &str) -> DBResult<StoredQuest> {
        let quest = self.quests.get(id).expect("couldn't find quest");
        Ok(quest.clone())
    }
    async fn delete_quest(&self, id: &str) -> DBResult<()> {
        self.quests.remove(id);
        Ok(())
    }
    async fn start_quest(&self, quest_id: &str, user_address: &str) -> DBResult<String> {
        let id = self.quest_instances.len().to_string();
        let quest_instance = QuestInstance {
            id,
            quest_id: quest_id.to_string(),
            user_address: user_address.to_string(),
            start_timestamp: 0
        };

        self.quest_instances.insert(id, quest_instance);

        Ok(id)
    }

    async fn get_quest_instance(&self, id: &str) -> DBResult<QuestInstance> {
        let quest_instance = self.quest_instances.get(id).expect("couldn't find quest instance");
        Ok(quest_instance.clone())
    }
    async fn get_user_quest_instances(&self, user_address: &str) -> DBResult<Vec<QuestInstance>> {
        Ok(vec![])
    }

    async fn add_event(&self, event: &AddEvent, quest_instance_id: &str) -> DBResult<()> {
        Ok(())
    }
    async fn get_events(&self, quest_instance_id: &str) -> DBResult<Vec<Event>> {
        Ok(vec![])
    }
}
