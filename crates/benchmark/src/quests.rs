use quests_protocol::quests::*;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

pub fn create_random_string(length: usize) -> String {
    let rand_string = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
    rand_string
}

fn random_action() -> Action {
    Action::custom(&create_random_string(10))
}

pub fn random_quest() -> Quest {
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
                description: None,
                action_items,
            };
            tasks.push(task);
        }
        let step = Step {
            id: format!("step-{step_id}"),
            description: create_random_string(50),
            tasks,
            on_complete_hook: None,
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

    Quest {
        name: create_random_string(10),
        description: create_random_string(100),
        definition: QuestDefinition { connections, steps },
    }
}

pub fn grab_some_apples() -> Quest {
    Quest {
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
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
                        description: None,
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: vec![Task {
                        id: "B_1".to_string(),
                        description: None,
                        action_items: vec![Action::location(Coordinates::new(13, 20))],
                    }],
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: vec![Task {
                        id: "C_1".to_string(),
                        description: None,
                        action_items: vec![Action::location(Coordinates::new(10, 24))],
                    }],
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: vec![Task {
                        id: "D_1".to_string(),
                        description: None,
                        action_items: vec![Action::location(Coordinates::new(40, 20))],
                    }],
                    on_complete_hook: None,
                    description: "".to_string(),
                },
            ],
        },
    }
}

pub fn grab_some_pies() -> Quest {
    Quest {
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
                        description: None,
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: vec![Task {
                        id: "B_1".to_string(),
                        description: None,
                        action_items: vec![Action::location(Coordinates::new(30, 20))],
                    }],
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: vec![Task {
                        id: "C_1".to_string(),
                        description: None,
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: vec![Task {
                        id: "D_1".to_string(),
                        description: None,
                        action_items: vec![Action::location(Coordinates::new(14, 20))],
                    }],
                    on_complete_hook: None,
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
        let valid = quest.is_valid();

        assert!(valid.is_ok());
    }
}
