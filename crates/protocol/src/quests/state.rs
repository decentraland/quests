use crate::{
    definitions::*,
    quests::{
        graph::{matches_action, QuestGraph},
        END_STEP_ID, START_STEP_ID,
    },
};
use std::collections::HashMap;

impl QuestState {
    pub fn is_completed(&self) -> bool {
        self.required_steps
            .iter()
            .all(|step| self.steps_completed.contains(step))
    }

    pub fn apply_event(&self, quest_graph: &QuestGraph, event: &Event) -> QuestState {
        // use this state to return
        let mut state = self.clone();
        let event_action = if let Some(action) = &event.action {
            action.clone()
        } else {
            return state;
        };

        for (step_id, step_content) in &self.current_steps {
            if step_content.to_dos.is_empty() {
                continue;
            }
            for (i, task) in step_content.to_dos.iter().enumerate() {
                match task
                    .action_items
                    .iter()
                    .position(|action| matches_action(action, &event_action))
                {
                    Some(matched_action_index) => {
                        state
                            .current_steps
                            .entry(step_id.to_string())
                            .and_modify(|step| {
                                step.to_dos[i].action_items.remove(matched_action_index);

                                if step.to_dos[i].action_items.is_empty() {
                                    step.tasks_completed.push(task.clone());
                                    step.to_dos.remove(i);
                                }
                            });
                    }
                    None => continue,
                }
            }
            if let Some(step) = state.current_steps.get(step_id) {
                if step.to_dos.is_empty() {
                    // remove step because it was completed
                    state.current_steps.remove(step_id);
                    state.steps_left -= 1;

                    // add next steps
                    let next_steps = quest_graph.next(step_id).unwrap_or_default();
                    next_steps.iter().for_each(|step_id| {
                        if step_id != END_STEP_ID {
                            let step_content = StepContent {
                                to_dos: quest_graph.tasks_by_step.get(step_id).unwrap().clone(),
                                ..Default::default()
                            };
                            state.current_steps.insert(step_id.clone(), step_content);
                        }
                    });

                    // mark just completed step as completed
                    state.steps_completed.push(step_id.clone());
                }
            }
        }

        state
    }
}

impl From<&QuestGraph> for QuestState {
    /// Returns the initial state of the Quest as it's not initialized
    fn from(graph: &QuestGraph) -> Self {
        let next_possible_steps = graph
            .next(START_STEP_ID)
            .unwrap_or_default()
            .iter()
            .map(|step| {
                (
                    step.clone(),
                    StepContent {
                        to_dos: graph.tasks_by_step.get(step).unwrap().clone(),
                        ..Default::default()
                    },
                )
            })
            .collect::<HashMap<String, StepContent>>();

        Self {
            current_steps: next_possible_steps,
            required_steps: graph.required_for_end().unwrap_or_default(),
            steps_left: graph.total_steps() as u32,
            steps_completed: Vec::default(),
        }
    }
}

pub fn get_state(quest: &Quest, events: Vec<Event>) -> QuestState {
    let quest_graph = QuestGraph::from(quest);
    let initial_state = (&quest_graph).into();
    events.iter().fold(initial_state, |state, event| {
        state.apply_event(&quest_graph, event)
    })
}

#[cfg(test)]
mod tests {
    use crate::quests::builders::Coordinates;

    use super::*;

    #[test]
    fn quest_graph_apply_event_task_simple_works() {
        let quest = Quest {
            id: "".to_string(),
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            creator_address: "0xB".to_string(),
            definition: Some(QuestDefinition {
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
                            description: "".to_string(),
                            action_items: vec![
                                Action::location(Coordinates::new(10, 10)),
                                Action::jump(Coordinates::new(10, 11)),
                            ],
                        }],
                    },
                    Step {
                        id: "A2".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "A2_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![Action::npc_interaction("NPC_IDEN")],
                        }],
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "B_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![Action::jump(Coordinates::new(20, 10))],
                        }],
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "C_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![Action::jump(Coordinates::new(20, 20))],
                        }],
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "D_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![Action::npc_interaction("OTHER_NPC")],
                        }],
                    },
                ],
            }),
            ..Default::default()
        };
        let quest_graph = QuestGraph::from(&quest);
        let mut events = vec![
            Event {
                // A1_1
                id: uuid::Uuid::new_v4().to_string(),
                address: "0xA".to_string(),
                action: Some(Action::location(Coordinates::new(10, 10))),
            },
            Event {
                // A2_1
                id: uuid::Uuid::new_v4().to_string(),
                address: "0xA".to_string(),
                action: Some(Action::jump(Coordinates::new(10, 11))),
            },
            Event {
                id: uuid::Uuid::new_v4().to_string(),
                address: "0xA".to_string(),
                action: Some(Action::jump(Coordinates::new(20, 10))),
            },
            Event {
                id: uuid::Uuid::new_v4().to_string(),
                address: "0xA".to_string(),
                action: Some(Action::jump(Coordinates::new(20, 20))),
            },
            Event {
                id: uuid::Uuid::new_v4().to_string(),
                address: "0xA".to_string(),
                action: Some(Action::npc_interaction("NPC_IDEN")),
            },
            Event {
                id: uuid::Uuid::new_v4().to_string(),
                address: "0xA".to_string(),
                action: Some(Action::npc_interaction("OTHER_NPC")),
            },
        ];

        let mut state = QuestState::from(&quest_graph);
        assert!(state.current_steps.contains_key("A1")); // branch 1
        assert!(state.current_steps.contains_key("A2")); // branch 2
        assert_eq!(state.current_steps.len(), 2);
        assert!(state.steps_completed.is_empty());
        assert_eq!(state.steps_left, 5);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.current_steps.contains_key("A1"));
        assert!(state.current_steps.contains_key("A2"));
        assert_eq!(state.current_steps.len(), 2);
        assert!(state.steps_completed.is_empty());
        assert_eq!(state.steps_left, 5);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));
        assert!(state.current_steps.get("A1").is_some());
        assert!(state
            .current_steps
            .get("A1")
            .unwrap()
            .tasks_completed
            .is_empty());

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.current_steps.contains_key("B"));
        assert!(state.current_steps.contains_key("A2"));
        assert_eq!(state.current_steps.len(), 2);
        assert!(state.steps_completed.contains(&"A1".to_string()));
        assert_eq!(state.steps_left, 4);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));
        assert!(state.current_steps.get("A1").is_none());

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.current_steps.contains_key("C"));
        assert!(state.current_steps.contains_key("A2"));
        assert_eq!(state.current_steps.len(), 2);
        assert!(state.steps_completed.contains(&"A1".to_string()));
        assert!(state.steps_completed.contains(&"B".to_string()));
        assert_eq!(state.steps_left, 3);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.current_steps.contains_key("A2"));
        assert_eq!(state.current_steps.len(), 1);
        assert!(state.steps_completed.contains(&"A1".to_string()));
        assert!(state.steps_completed.contains(&"B".to_string()));
        assert!(state.steps_completed.contains(&"C".to_string()));
        assert_eq!(state.steps_left, 2);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.current_steps.contains_key("D"));
        assert_eq!(state.current_steps.len(), 1);
        assert!(state.steps_completed.contains(&"A1".to_string()));
        assert!(state.steps_completed.contains(&"A2".to_string()));
        assert!(state.steps_completed.contains(&"B".to_string()));
        assert!(state.steps_completed.contains(&"C".to_string()));
        assert_eq!(state.steps_left, 1);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.current_steps.is_empty());
        assert!(state.steps_completed.contains(&"A1".to_string()));
        assert!(state.steps_completed.contains(&"A2".to_string()));
        assert!(state.steps_completed.contains(&"B".to_string()));
        assert!(state.steps_completed.contains(&"C".to_string()));
        assert!(state.steps_completed.contains(&"D".to_string()));
        assert_eq!(state.steps_left, 0);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));
        assert!(state.is_completed());
    }

    #[test]
    fn quest_graph_apply_event_task_multiple_works() {
        let quest = Quest {
            id: "".to_string(),
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            creator_address: "0xB".to_string(),
            definition: Some(QuestDefinition {
                connections: vec![Connection::new("A", "B"), Connection::new("B", "C")],
                steps: vec![
                    Step {
                        id: "A".to_string(),
                        description: "".to_string(),
                        tasks: vec![
                            Task {
                                id: "A_1".to_string(),
                                description: "".to_string(),
                                action_items: vec![
                                    Action::jump(Coordinates::new(10, 10)),
                                    Action::location(Coordinates::new(15, 10)),
                                ],
                            },
                            Task {
                                id: "A_2".to_string(),
                                description: "".to_string(),
                                action_items: vec![
                                    Action::npc_interaction("NPC_ID"),
                                    Action::location(Coordinates::new(15, 14)),
                                ],
                            },
                        ],
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: vec![
                            Task {
                                id: "B_1".to_string(),
                                description: "".to_string(),
                                action_items: vec![
                                    Action::jump(Coordinates::new(10, 20)),
                                    Action::location(Coordinates::new(23, 14)),
                                ],
                            },
                            Task {
                                id: "B_2".to_string(),
                                description: "".to_string(),
                                action_items: vec![
                                    Action::custom("a"),
                                    Action::location(Coordinates::new(40, 10)),
                                ],
                            },
                        ],
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "C_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![Action::jump(Coordinates::new(20, 20))],
                        }],
                    },
                ],
            }),
            ..Default::default()
        };

        let quest_graph = QuestGraph::from(&quest);
        let mut events = vec![
            Event {
                id: uuid::Uuid::new_v4().to_string(),
                address: "0xA".to_string(),
                action: Some(Action::jump(Coordinates::new(10, 10))),
            },
            Event {
                id: uuid::Uuid::new_v4().to_string(),
                address: "0xA".to_string(),
                action: Some(Action::location(Coordinates::new(15, 10))),
            },
            Event {
                id: uuid::Uuid::new_v4().to_string(),
                address: "0xA".to_string(),
                action: Some(Action::npc_interaction("NPC_ID")),
            },
            Event {
                id: uuid::Uuid::new_v4().to_string(),
                address: "0xA".to_string(),
                action: Some(Action::location(Coordinates::new(15, 14))),
            },
            Event {
                id: uuid::Uuid::new_v4().to_string(),
                address: "0xA".to_string(),
                action: Some(Action::jump(Coordinates::new(10, 20))),
            },
            Event {
                id: uuid::Uuid::new_v4().to_string(),
                address: "0xA".to_string(),
                action: Some(Action::location(Coordinates::new(23, 14))),
            },
            Event {
                id: uuid::Uuid::new_v4().to_string(),
                address: "0xA".to_string(),
                action: Some(Action::custom("a")),
            },
            Event {
                id: uuid::Uuid::new_v4().to_string(),
                address: "0xA".to_string(),
                action: Some(Action::location(Coordinates::new(40, 10))),
            },
            Event {
                id: uuid::Uuid::new_v4().to_string(),
                address: "0xA".to_string(),
                action: Some(Action::jump(Coordinates::new(20, 20))),
            },
        ];
        let mut state = QuestState::from(&quest_graph);
        assert!(state.current_steps.contains_key("A"));

        let tasks = &state.current_steps.get("A").unwrap().to_dos;
        assert_eq!(tasks.len(), 2);
        assert_eq!(
            tasks.get(0).unwrap(),
            &Task {
                id: "A_1".to_string(),
                description: "".to_string(),
                action_items: vec![
                    Action::jump(Coordinates::new(10, 10)),
                    Action::location(Coordinates::new(15, 10)),
                ],
            }
        );
        assert_eq!(
            tasks.get(1).unwrap(),
            &Task {
                id: "A_2".to_string(),
                description: "".to_string(),
                action_items: vec![
                    Action::npc_interaction("NPC_ID"),
                    Action::location(Coordinates::new(15, 14)),
                ],
            }
        );

        assert_eq!(state.current_steps.len(), 1);
        assert!(state.steps_completed.is_empty());
        assert_eq!(state.steps_left, 3);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert_eq!(state.required_steps.len(), 1);

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.current_steps.contains_key("A"));
        assert_eq!(state.current_steps.len(), 1);

        let tasks = &state.current_steps.get("A").unwrap().to_dos;
        assert_eq!(tasks.len(), 2);
        assert_eq!(
            tasks.get(0).unwrap(),
            &Task {
                id: "A_1".to_string(),
                description: "".to_string(),
                action_items: vec![Action::location(Coordinates::new(15, 10))],
            }
        );
        assert_eq!(
            tasks.get(1).unwrap(),
            &Task {
                id: "A_2".to_string(),
                description: "".to_string(),
                action_items: vec![
                    Action::npc_interaction("NPC_ID"),
                    Action::location(Coordinates::new(15, 14)),
                ],
            }
        );
        assert!(state
            .current_steps
            .get("A")
            .unwrap()
            .tasks_completed
            .is_empty());

        assert_eq!(state.steps_left, 3);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert_eq!(state.required_steps.len(), 1);

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.current_steps.contains_key("A"));
        assert_eq!(state.current_steps.len(), 1);
        let tasks = &state.current_steps.get("A").unwrap().to_dos;
        assert_eq!(tasks.len(), 1);
        assert_eq!(
            tasks.get(0).unwrap(),
            &Task {
                id: "A_2".to_string(),
                description: "".to_string(),
                action_items: vec![
                    Action::npc_interaction("NPC_ID"),
                    Action::location(Coordinates::new(15, 14)),
                ],
            }
        );
        assert!(state.steps_completed.is_empty());
        assert_eq!(state.steps_left, 3);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert_eq!(state.required_steps.len(), 1);
        assert_eq!(
            state.current_steps.get("A").unwrap().tasks_completed.len(),
            1
        );
        assert!(state
            .current_steps
            .get("A")
            .unwrap()
            .tasks_completed
            .iter()
            .any(|task| task.id == "A_1"));

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.current_steps.contains_key("A"));
        assert_eq!(state.current_steps.len(), 1);

        let subtasks = &state.current_steps.get("A").unwrap().to_dos;
        assert_eq!(subtasks.len(), 1);
        assert_eq!(
            subtasks.get(0).unwrap(),
            &Task {
                id: "A_2".to_string(),
                description: "".to_string(),
                action_items: vec![Action::location(Coordinates::new(15, 14)),],
            }
        );
        assert!(state.steps_completed.is_empty());
        assert_eq!(state.steps_left, 3);
        assert_eq!(state.required_steps.len(), 1);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert_eq!(
            state.current_steps.get("A").unwrap().tasks_completed.len(),
            1
        );
        assert!(state
            .current_steps
            .get("A")
            .unwrap()
            .tasks_completed
            .iter()
            .any(|task| task.id == "A_1"));

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.current_steps.contains_key("B"));
        assert_eq!(state.current_steps.len(), 1);
        assert!(!state.current_steps.contains_key("A"));

        let tasks = &state.current_steps.get("B").unwrap().to_dos;
        assert_eq!(tasks.len(), 2);
        assert_eq!(
            tasks.get(0).unwrap(),
            &Task {
                id: "B_1".to_string(),
                description: "".to_string(),
                action_items: vec![
                    Action::jump(Coordinates::new(10, 20)),
                    Action::location(Coordinates::new(23, 14)),
                ],
            },
        );
        assert_eq!(
            tasks.get(1).unwrap(),
            &Task {
                id: "B_2".to_string(),
                description: "".to_string(),
                action_items: vec![
                    Action::custom("a"),
                    Action::location(Coordinates::new(40, 10)),
                ],
            },
        );

        assert!(state.steps_completed.contains(&"A".to_string()));
        assert_eq!(state.steps_left, 2);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert_eq!(state.required_steps.len(), 1);

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.current_steps.contains_key("B"));
        assert_eq!(state.current_steps.len(), 1);

        let tasks = &state.current_steps.get("B").unwrap().to_dos;
        assert_eq!(tasks.len(), 2);
        assert_eq!(
            tasks.get(0).unwrap(),
            &Task {
                id: "B_1".to_string(),
                description: "".to_string(),
                action_items: vec![Action::location(Coordinates::new(23, 14)),],
            },
        );
        assert_eq!(
            tasks.get(1).unwrap(),
            &Task {
                id: "B_2".to_string(),
                description: "".to_string(),
                action_items: vec![
                    Action::custom("a"),
                    Action::location(Coordinates::new(40, 10)),
                ],
            },
        );

        assert!(state.steps_completed.contains(&"A".to_string()));
        assert_eq!(state.steps_left, 2);
        assert!(state.required_steps.contains(&"C".to_string()));

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.current_steps.contains_key("B"));
        assert_eq!(state.current_steps.len(), 1);
        let tasks = &state.current_steps.get("B").unwrap().to_dos;
        assert_eq!(tasks.len(), 1);
        assert_eq!(
            tasks.get(0).unwrap(),
            &Task {
                id: "B_2".to_string(),
                description: "".to_string(),
                action_items: vec![
                    Action::custom("a"),
                    Action::location(Coordinates::new(40, 10)),
                ],
            },
        );

        assert!(state.steps_completed.contains(&"A".to_string()));
        assert_eq!(state.steps_left, 2);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert_eq!(state.required_steps.len(), 1);
        assert_eq!(
            state.current_steps.get("B").unwrap().tasks_completed.len(),
            1
        );
        assert!(state
            .current_steps
            .get("B")
            .unwrap()
            .tasks_completed
            .iter()
            .any(|task| task.id == "B_1"));

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.current_steps.contains_key("B"));
        assert_eq!(state.current_steps.len(), 1);
        let tasks = &state.current_steps.get("B").unwrap().to_dos;
        assert_eq!(tasks.len(), 1);
        assert_eq!(
            tasks.get(0).unwrap(),
            &Task {
                id: "B_2".to_string(),
                description: "".to_string(),
                action_items: vec![Action::location(Coordinates::new(40, 10)),],
            },
        );

        assert!(state.steps_completed.contains(&"A".to_string()));
        assert_eq!(state.steps_left, 2);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert_eq!(state.required_steps.len(), 1);
        assert_eq!(
            state.current_steps.get("B").unwrap().tasks_completed.len(),
            1
        );
        assert!(state
            .current_steps
            .get("B")
            .unwrap()
            .tasks_completed
            .iter()
            .any(|task| task.id == "B_1"));

        assert!(!state
            .current_steps
            .get("B")
            .unwrap()
            .tasks_completed
            .iter()
            .any(|task| task.id == "B_2"));

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.current_steps.contains_key("C"));
        assert_eq!(state.current_steps.len(), 1);
        assert!(state.steps_completed.contains(&"A".to_string()));
        assert!(state.steps_completed.contains(&"B".to_string()));
        assert_eq!(state.steps_left, 1);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert_eq!(state.required_steps.len(), 1);
        assert!(state.current_steps.get("B").is_none());

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert_eq!(state.current_steps.len(), 0);
        assert!(state.steps_completed.contains(&"A".to_string()));
        assert!(state.steps_completed.contains(&"B".to_string()));
        assert!(state.steps_completed.contains(&"C".to_string()));
        assert_eq!(state.steps_left, 0);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert_eq!(state.required_steps.len(), 1);
        assert_eq!(state.current_steps.len(), 0);
        assert!(state.is_completed())
    }
}
