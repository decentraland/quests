use quests_protocol::definitions::*;
use quests_protocol::quests::*;

pub fn grab_some_apples() -> Quest {
    Quest {
        id: "3e9a8bbf-2223-4f51-b7e5-660d35cedef4".to_string(),
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
                        description: "A_1 Desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "A Step Description".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: vec![Task {
                        id: "B_1".to_string(),
                        description: "B_1 DESC".to_string(),
                        action_items: vec![Action::location(Coordinates::new(13, 20))],
                    }],
                    description: "B Desc".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: vec![Task {
                        id: "C_1".to_string(),
                        description: "C_1 Desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 24))],
                    }],
                    description: "C Desc".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: vec![Task {
                        id: "D_1".to_string(),
                        description: "D_1 Desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(40, 20))],
                    }],
                    description: "D desc".to_string(),
                },
            ],
        }),
        ..Default::default()
    }
}

#[allow(dead_code)]
pub fn grab_some_pies() -> Quest {
    Quest {
        id: "1e9a8bbf-2223-4f51-b7e5-660d35cedef4".to_string(),
        name: "QUEST-2".to_string(),
        description: "Grab some pies".to_string(),
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
                        description: "A_1 desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "A desc".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: vec![Task {
                        id: "B_1".to_string(),
                        description: "B_1 desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(30, 20))],
                    }],
                    description: "B Desc".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: vec![Task {
                        id: "C_1".to_string(),
                        description: "C_1 desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                    description: "C desc".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: vec![Task {
                        id: "D_1".to_string(),
                        description: "D_1 desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(14, 20))],
                    }],
                    description: "D Desc".to_string(),
                },
            ],
        }),
        ..Default::default()
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
        id: "2e9a8bbf-2223-4f51-b7e5-660d35cedef4".to_string(),
        name: "One step quest".to_string(),
        description: "Jump".to_string(),
        creator_address: "0xB".to_string(),
        definition: Some(QuestDefinition {
            connections: vec![],
            steps: vec![Step {
                id: "Jump".to_string(),
                tasks: vec![Task {
                    id: "Jump once".to_string(),
                    description: "Jump once desc".to_string(),
                    action_items: vec![action.clone()],
                }],
                description: "Jump desc".to_string(),
            }],
        }),
        ..Default::default()
    };
    (quest, action)
}
