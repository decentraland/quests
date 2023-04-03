use quests_protocol::{quest_graph::QuestGraph, quests::*};

fn main() {
    println!("Quests definitions:");

    let branched_quest = Quest {
        name: "CUSTOM_QUEST".to_string(),
        description: "".to_string(),
        definition: QuestDefinition {
            connections: vec![
                Connection::new("A1", "B"),
                Connection::new("B", "C"),
                Connection::new("A2", "D"),
            ],
            steps: vec![
                Step {
                    id: "A1".to_string(),
                    description: "".to_string(),
                    tasks: vec![Task {
                        id: "A1_1".to_string(),
                        description: None,
                        action_items: vec![
                            Action::location(Coordinates::new(10, 10)),
                            Action::jump(Coordinates::new(10, 11)),
                        ],
                    }],
                    on_complete_hook: None,
                },
                Step {
                    id: "A2".to_string(),
                    description: "".to_string(),
                    tasks: vec![Task {
                        id: "A2_1".to_string(),
                        description: None,
                        action_items: vec![Action::npc_interaction("NPC_IDEN")],
                    }],
                    on_complete_hook: None,
                },
                Step {
                    id: "B".to_string(),
                    description: "".to_string(),
                    tasks: vec![Task {
                        id: "B_1".to_string(),
                        description: None,
                        action_items: vec![Action::jump(Coordinates::new(20, 10))],
                    }],
                    on_complete_hook: None,
                },
                Step {
                    id: "C".to_string(),
                    description: "".to_string(),
                    tasks: vec![Task {
                        id: "C_1".to_string(),
                        description: None,
                        action_items: vec![Action::jump(Coordinates::new(20, 20))],
                    }],
                    on_complete_hook: None,
                },
                Step {
                    id: "D".to_string(),
                    description: "".to_string(),
                    tasks: vec![Task {
                        id: "D_1".to_string(),
                        description: None,
                        action_items: vec![Action::npc_interaction("OTHER_NPC")],
                    }],
                    on_complete_hook: None,
                },
            ],
        },
    };

    print_quest(&branched_quest);

    let quest_graph = QuestGraph::from(&branched_quest);
    println!("{}", quest_graph.get_quest_draw());
}

fn print_quest(quest: &Quest) {
    println!("{}", serde_json::to_string(quest).unwrap());
}
