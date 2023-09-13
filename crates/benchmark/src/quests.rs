use quests_protocol::definitions::*;
use quests_protocol::quests::*;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Serialize;

pub fn create_random_string(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

fn random_action() -> Action {
    Action::custom(&create_random_string(10))
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateQuestRequest {
    name: String,
    description: String,
    definition: QuestDefinition,
}

pub fn random_quest() -> CreateQuestRequest {
    let mut connections = vec![];
    let mut steps = vec![];
    let mut rng = rand::thread_rng();
    let quest_steps = rng.gen_range(3..20);
    for step_id in 0..quest_steps {
        let mut tasks = vec![];
        let step_tasks = rng.gen_range(1..=3);
        for task_id in 0..step_tasks {
            let mut action_items = vec![];
            let task_actions = rng.gen_range(1..=3);
            for _ in 0..task_actions {
                action_items.push(random_action());
            }
            let task = Task {
                id: format!("step-{step_id}-task-{task_id}"),
                description: format!("step-{step_id}-task-{task_id}-description"),
                action_items,
            };
            tasks.push(task);
        }
        let step = Step {
            id: format!("step-{step_id}"),
            description: create_random_string(50),
            tasks,
        };
        steps.push(step);
    }

    // connect steps
    for i in 0..steps.len() {
        if i + 1 < steps.len() {
            let node_a = &steps[i];
            let node_b = &steps[i + 1];
            connections.push(Connection::new(&node_a.id, &node_b.id));
        }
    }

    CreateQuestRequest {
        name: create_random_string(10),
        description: create_random_string(100),
        definition: QuestDefinition { connections, steps },
    }
}

pub fn grab_some_apples() -> Quest {
    Quest {
        id: "8e9a8bbf-2223-4f51-b7e5-660d35cedef4".to_string(),
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
        creator_address: "0xB".to_string(),
        definition: Some(QuestDefinition {
            connections: vec![
                Connection::new("A", "B"),
                Connection::new("B", "C"),
                Connection::new("C", "D"),
            ],
            steps: vec![
                Step {
                    id: "A".to_string(),
                    tasks: vec![Task {
                        id: "A_1".to_string(),
                        description: "".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: vec![Task {
                        id: "B_1".to_string(),
                        description: "".to_string(),
                        action_items: vec![Action::location(Coordinates::new(13, 20))],
                    }],
                    description: "".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: vec![Task {
                        id: "C_1".to_string(),
                        description: "".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 24))],
                    }],
                    description: "".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: vec![Task {
                        id: "D_1".to_string(),
                        description: "".to_string(),
                        action_items: vec![Action::location(Coordinates::new(40, 20))],
                    }],
                    description: "".to_string(),
                },
            ],
        }),
        ..Default::default()
    }
}

pub fn grab_some_pies() -> CreateQuestRequest {
    CreateQuestRequest {
        name: "QUEST-2".to_string(),
        description: "Grab some pies".to_string(),
        definition: QuestDefinition {
            connections: vec![
                Connection::new("A", "B"),
                Connection::new("B", "C"),
                Connection::new("C", "D"),
            ],
            steps: vec![
                Step {
                    id: "A".to_string(),
                    tasks: vec![Task {
                        id: "A_1".to_string(),
                        description: "".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: vec![Task {
                        id: "B_1".to_string(),
                        description: "".to_string(),
                        action_items: vec![Action::location(Coordinates::new(30, 20))],
                    }],
                    description: "".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: vec![Task {
                        id: "C_1".to_string(),
                        description: "".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: vec![Task {
                        id: "D_1".to_string(),
                        description: "".to_string(),
                        action_items: vec![Action::location(Coordinates::new(14, 20))],
                    }],
                    description: "".to_string(),
                },
            ],
        },
    }
}

mod tests {

    #[test]
    fn random_quest_is_valid() {
        use super::random_quest;
        let quest = random_quest();

        println!("Quest {quest:#?}");
        let valid = quest.definition.is_valid();

        assert!(valid.is_ok());
    }
}
