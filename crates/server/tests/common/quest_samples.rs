use quests_definitions::quests::*;

pub fn grab_some_apples() -> Quest {
    Quest {
        name: "QUEST-1".to_string(),
        description: "Grab some apples".to_string(),
        definition: QuestDefinition {
            connections: vec![
                ("A".to_string(), "B".to_string()),
                ("B".to_string(), "C".to_string()),
                ("C".to_string(), "D".to_string()),
            ],
            steps: vec![
                Step {
                    id: "A".to_string(),
                    tasks: vec![Task {
                        id: "A_1".to_string(),
                        description: None,
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(10, 20),
                        }],
                    }],
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: vec![Task {
                        id: "B_1".to_string(),
                        description: None,
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(13, 20),
                        }],
                    }],
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: vec![Task {
                        id: "C_1".to_string(),
                        description: None,
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(10, 24),
                        }],
                    }],
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: vec![Task {
                        id: "D_1".to_string(),
                        description: None,
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(40, 20),
                        }],
                    }],
                    on_complete_hook: None,
                    description: "".to_string(),
                },
            ],
        },
    }
}

#[allow(dead_code)]
pub fn grab_some_pies() -> Quest {
    Quest {
        name: "QUEST-2".to_string(),
        description: "Grab some pies".to_string(),
        definition: QuestDefinition {
            connections: vec![
                ("A".to_string(), "B".to_string()),
                ("B".to_string(), "C".to_string()),
                ("C".to_string(), "D".to_string()),
            ],
            steps: vec![
                Step {
                    id: "A".to_string(),
                    tasks: vec![Task {
                        id: "A_1".to_string(),
                        description: None,
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(10, 20),
                        }],
                    }],
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: vec![Task {
                        id: "B_1".to_string(),
                        description: None,
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(30, 20),
                        }],
                    }],
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: vec![Task {
                        id: "C_1".to_string(),
                        description: None,
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(10, 23),
                        }],
                    }],
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: vec![Task {
                        id: "D_1".to_string(),
                        description: None,
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(14, 20),
                        }],
                    }],
                    on_complete_hook: None,
                    description: "".to_string(),
                },
            ],
        },
    }
}
