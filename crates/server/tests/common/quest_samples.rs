use quests_protocol::definitions::*;
use quests_protocol::quests::*;

pub fn grab_some_apples() -> Quest {
    Quest {
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
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
    }
}

#[allow(dead_code)]
pub fn grab_some_pies() -> Quest {
    Quest {
        name: "QUEST-2".to_string(),
        description: "Grab some pies".to_string(),
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
        }),
    }
}

#[allow(dead_code)]
pub fn one_step_quest() -> (Quest, Action) {
    use std::collections::HashMap;
    let action = Action {
        r#type: "Jump".to_string(),
        parameters: HashMap::new(),
    };
    let quest = Quest {
        name: "One step quest".to_string(),
        description: "Jump".to_string(),
        definition: Some(QuestDefinition {
            connections: vec![],
            steps: vec![Step {
                id: "Jump".to_string(),
                tasks: vec![Task {
                    id: "Jump once".to_string(),
                    description: "".to_string(),
                    action_items: vec![action.clone()],
                }],
                description: "".to_string(),
            }],
        }),
    };
    (quest, action)
}
