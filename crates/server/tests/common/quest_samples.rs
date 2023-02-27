use quests_definitions::quests::*;

pub fn simple_quest() -> Quest {
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
                    tasks: Tasks::Single {
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(10, 20),
                        }],
                    },
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "B".to_string(),
                    tasks: Tasks::Single {
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(13, 20),
                        }],
                    },
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "C".to_string(),
                    tasks: Tasks::Single {
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(10, 24),
                        }],
                    },
                    on_complete_hook: None,
                    description: "".to_string(),
                },
                Step {
                    id: "D".to_string(),
                    tasks: Tasks::Single {
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(40, 20),
                        }],
                    },
                    on_complete_hook: None,
                    description: "".to_string(),
                },
            ],
        },
    }
}
