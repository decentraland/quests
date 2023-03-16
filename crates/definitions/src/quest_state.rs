use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    quest_graph::{matches_action, QuestGraph},
    quests::{Event, Quest, StepID, Task, END_STEP_ID, START_STEP_ID},
};

#[derive(Serialize, Deserialize)]
pub struct QuestUpdate {
    pub state: QuestState,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct QuestState {
    /// Current steps
    pub current_steps: HashMap<StepID, StepContent>,
    /// Steps left to complete the Quest
    pub steps_left: u32,
    /// Required Steps for END
    pub required_steps: Vec<StepID>,
    /// Quest Steps Completed
    pub steps_completed: HashSet<StepID>,
}

impl QuestState {
    pub fn is_completed(&self) -> bool {
        self.required_steps
            .iter()
            .all(|step| self.steps_completed.contains(step))
    }

    pub fn apply_event(&self, quest_graph: &QuestGraph, event: &Event) -> QuestState {
        // use this state to return
        let state = self.clone();

        // We do many clones because we don't want to mutate the state given directly. And also, we don't want to keep a state in the QuestGraph
        // We clone the next possible steps in order to mutate this instance for the new state
        let mut current_steps_cloned = state.current_steps.clone();
        // We clone the current completed steps in order to add the new ones for the event given
        let mut steps_completed = state.steps_completed.clone();

        for (step_id, step_content) in state.current_steps {
            if step_content.to_dos.is_empty() {
                continue;
            }
            let mut to_dos_cloned = step_content.to_dos.clone();
            let mut tasks_completed_cloned = step_content.tasks_completed.clone();
            for (i, task) in step_content.to_dos.iter().enumerate() {
                let mut actions_items_cloned = task.action_items.clone();
                match task
                    .action_items
                    .iter()
                    .position(|action| matches_action((action.clone(), event.action.clone())))
                {
                    Some(matched_action_index) => {
                        actions_items_cloned.remove(matched_action_index);

                        if actions_items_cloned.is_empty() {
                            tasks_completed_cloned.insert(task.id.clone());
                            to_dos_cloned.remove(i);
                        } else {
                            to_dos_cloned[i] = Task {
                                id: task.id.clone(),
                                description: task.description.clone(),
                                action_items: actions_items_cloned,
                            };
                        }
                    }
                    None => continue,
                }
            }

            if to_dos_cloned.is_empty() {
                // remove step because it was completed
                current_steps_cloned.remove(&step_id);

                // add next steps
                let next_steps = quest_graph.next(&step_id).unwrap_or_default();
                next_steps.iter().for_each(|step_id| {
                    if step_id != END_STEP_ID {
                        let step_content = StepContent {
                            to_dos: quest_graph.tasks_by_step.get(step_id).unwrap().clone(),
                            ..Default::default()
                        };
                        current_steps_cloned.insert(step_id.clone(), step_content);
                    }
                });

                // mark just completed step as completed
                steps_completed.insert(step_id.clone());
            } else {
                let step = current_steps_cloned.get_mut(&step_id).unwrap();
                step.to_dos = to_dos_cloned;
                step.tasks_completed = tasks_completed_cloned;
            }
        }

        QuestState {
            current_steps: current_steps_cloned,
            steps_left: (quest_graph.total_steps() - steps_completed.len()) as u32,
            required_steps: state.required_steps,
            steps_completed,
        }
    }
}

impl From<&QuestGraph> for QuestState {
    /// Returns the initial state of the Quest as it's not initialized
    fn from(value: &QuestGraph) -> Self {
        let next_possible_steps = value
            .next(START_STEP_ID)
            .unwrap_or_default()
            .iter()
            .map(|step| {
                (
                    step.clone(),
                    StepContent {
                        to_dos: value.tasks_by_step.get(step).unwrap().clone(),
                        ..Default::default()
                    },
                )
            })
            .collect::<HashMap<String, StepContent>>();

        Self {
            current_steps: next_possible_steps,
            required_steps: value.required_for_end().unwrap_or_default(),
            steps_left: value.total_steps() as u32,
            steps_completed: HashSet::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Deserialize, Serialize)]
pub struct StepContent {
    pub to_dos: Vec<Task>,
    /// Tasks completed. Inner Step tasks
    ///
    /// String in key refers to Task ID
    ///
    pub tasks_completed: HashSet<String>,
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
    use crate::quests::{Action, Coordinates, QuestDefinition, Step};

    use super::*;

    #[test]
    fn quest_graph_apply_event_task_simple_works() {
        let quest = Quest {
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: QuestDefinition {
                connections: vec![
                    ("A1".to_string(), "B".to_string()),
                    ("B".to_string(), "C".to_string()),
                    ("A2".to_string(), "D".to_string()),
                ],
                steps: vec![
                    Step {
                        id: "A1".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "A1_1".to_string(),
                            description: None,
                            action_items: vec![
                                Action::Location {
                                    coordinates: Coordinates(10, 10),
                                },
                                Action::Jump {
                                    coordinates: Coordinates(10, 11),
                                },
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
                            action_items: vec![Action::NPCInteraction {
                                npc_id: "NPC_IDEN".to_string(),
                            }],
                        }],
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "B_1".to_string(),
                            description: None,
                            action_items: vec![Action::Jump {
                                coordinates: Coordinates(20, 10),
                            }],
                        }],
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "C_1".to_string(),
                            description: None,
                            action_items: vec![Action::Jump {
                                coordinates: Coordinates(20, 20),
                            }],
                        }],
                        on_complete_hook: None,
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "D_1".to_string(),
                            description: None,
                            action_items: vec![Action::NPCInteraction {
                                npc_id: "OTHER_NPC".to_string(),
                            }],
                        }],
                        on_complete_hook: None,
                    },
                ],
            },
        };
        let quest_graph = QuestGraph::from(&quest);
        let mut events = vec![
            Event {
                // A1_1
                address: "0xA".to_string(),
                action: Action::Location {
                    coordinates: Coordinates(10, 10),
                },
            },
            Event {
                // A2_1
                address: "0xA".to_string(),
                action: Action::Jump {
                    coordinates: Coordinates(10, 11),
                },
            },
            Event {
                address: "0xA".to_string(),
                action: Action::Jump {
                    coordinates: Coordinates(20, 10),
                },
            },
            Event {
                address: "0xA".to_string(),
                action: Action::Jump {
                    coordinates: Coordinates(20, 20),
                },
            },
            Event {
                address: "0xA".to_string(),
                action: Action::NPCInteraction {
                    npc_id: "NPC_IDEN".to_string(),
                },
            },
            Event {
                address: "0xA".to_string(),
                action: Action::NPCInteraction {
                    npc_id: "OTHER_NPC".to_string(),
                },
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
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: QuestDefinition {
                connections: vec![
                    ("A".to_string(), "B".to_string()),
                    ("B".to_string(), "C".to_string()),
                ],
                steps: vec![
                    Step {
                        id: "A".to_string(),
                        description: "".to_string(),
                        tasks: vec![
                            Task {
                                id: "A_1".to_string(),
                                description: None,
                                action_items: vec![
                                    Action::Jump {
                                        coordinates: Coordinates(10, 10),
                                    },
                                    Action::Location {
                                        coordinates: Coordinates(15, 10),
                                    },
                                ],
                            },
                            Task {
                                id: "A_2".to_string(),
                                description: None,
                                action_items: vec![
                                    Action::NPCInteraction {
                                        npc_id: "NPC_ID".to_string(),
                                    },
                                    Action::Location {
                                        coordinates: Coordinates(15, 14),
                                    },
                                ],
                            },
                        ],
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: vec![
                            Task {
                                id: "B_1".to_string(),
                                description: None,
                                action_items: vec![
                                    Action::Jump {
                                        coordinates: Coordinates(10, 20),
                                    },
                                    Action::Location {
                                        coordinates: Coordinates(23, 14),
                                    },
                                ],
                            },
                            Task {
                                id: "B_2".to_string(),
                                description: None,
                                action_items: vec![
                                    Action::Custom {
                                        id: "a".to_string(),
                                    },
                                    Action::Location {
                                        coordinates: Coordinates(40, 10),
                                    },
                                ],
                            },
                        ],
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "C_1".to_string(),
                            description: None,
                            action_items: vec![Action::Jump {
                                coordinates: Coordinates(20, 20),
                            }],
                        }],
                        on_complete_hook: None,
                    },
                ],
            },
        };

        let quest_graph = QuestGraph::from(&quest);
        let mut events = vec![
            Event {
                address: "0xA".to_string(),
                action: Action::Jump {
                    coordinates: Coordinates(10, 10),
                },
            },
            Event {
                address: "0xA".to_string(),
                action: Action::Location {
                    coordinates: Coordinates(15, 10),
                },
            },
            Event {
                address: "0xA".to_string(),
                action: Action::NPCInteraction {
                    npc_id: "NPC_ID".to_string(),
                },
            },
            Event {
                address: "0xA".to_string(),
                action: Action::Location {
                    coordinates: Coordinates(15, 14),
                },
            },
            Event {
                address: "0xA".to_string(),
                action: Action::Jump {
                    coordinates: Coordinates(10, 20),
                },
            },
            Event {
                address: "0xA".to_string(),
                action: Action::Location {
                    coordinates: Coordinates(23, 14),
                },
            },
            Event {
                address: "0xA".to_string(),
                action: Action::Custom {
                    id: "a".to_string(),
                },
            },
            Event {
                address: "0xA".to_string(),
                action: Action::Location {
                    coordinates: Coordinates(40, 10),
                },
            },
            Event {
                address: "0xA".to_string(),
                action: Action::Jump {
                    coordinates: Coordinates(20, 20),
                },
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
                description: None,
                action_items: vec![
                    Action::Jump {
                        coordinates: Coordinates(10, 10),
                    },
                    Action::Location {
                        coordinates: Coordinates(15, 10),
                    },
                ]
            }
        );
        assert_eq!(
            tasks.get(1).unwrap(),
            &Task {
                id: "A_2".to_string(),
                description: None,
                action_items: vec![
                    Action::NPCInteraction {
                        npc_id: "NPC_ID".to_string(),
                    },
                    Action::Location {
                        coordinates: Coordinates(15, 14),
                    },
                ]
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
                description: None,
                action_items: vec![Action::Location {
                    coordinates: Coordinates(15, 10),
                },]
            }
        );
        assert_eq!(
            tasks.get(1).unwrap(),
            &Task {
                id: "A_2".to_string(),
                description: None,
                action_items: vec![
                    Action::NPCInteraction {
                        npc_id: "NPC_ID".to_string(),
                    },
                    Action::Location {
                        coordinates: Coordinates(15, 14),
                    },
                ]
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
                description: None,
                action_items: vec![
                    Action::NPCInteraction {
                        npc_id: "NPC_ID".to_string(),
                    },
                    Action::Location {
                        coordinates: Coordinates(15, 14),
                    },
                ]
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
            .contains("A_1"));

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.current_steps.contains_key("A"));
        assert_eq!(state.current_steps.len(), 1);

        let subtasks = &state.current_steps.get("A").unwrap().to_dos;
        assert_eq!(subtasks.len(), 1);
        assert_eq!(
            subtasks.get(0).unwrap(),
            &Task {
                id: "A_2".to_string(),
                description: None,
                action_items: vec![Action::Location {
                    coordinates: Coordinates(15, 14),
                },]
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
            .contains("A_1"));

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
                description: None,
                action_items: vec![
                    Action::Jump {
                        coordinates: Coordinates(10, 20),
                    },
                    Action::Location {
                        coordinates: Coordinates(23, 14),
                    },
                ],
            },
        );
        assert_eq!(
            tasks.get(1).unwrap(),
            &Task {
                id: "B_2".to_string(),
                description: None,
                action_items: vec![
                    Action::Custom {
                        id: "a".to_string(),
                    },
                    Action::Location {
                        coordinates: Coordinates(40, 10),
                    },
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
                description: None,
                action_items: vec![Action::Location {
                    coordinates: Coordinates(23, 14),
                },],
            },
        );
        assert_eq!(
            tasks.get(1).unwrap(),
            &Task {
                id: "B_2".to_string(),
                description: None,
                action_items: vec![
                    Action::Custom {
                        id: "a".to_string(),
                    },
                    Action::Location {
                        coordinates: Coordinates(40, 10),
                    },
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
                description: None,
                action_items: vec![
                    Action::Custom {
                        id: "a".to_string(),
                    },
                    Action::Location {
                        coordinates: Coordinates(40, 10),
                    },
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
            .contains("B_1"));

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.current_steps.contains_key("B"));
        assert_eq!(state.current_steps.len(), 1);
        let tasks = &state.current_steps.get("B").unwrap().to_dos;
        assert_eq!(tasks.len(), 1);
        assert_eq!(
            tasks.get(0).unwrap(),
            &Task {
                id: "B_2".to_string(),
                description: None,
                action_items: vec![Action::Location {
                    coordinates: Coordinates(40, 10),
                },],
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
            .contains("B_1"));
        assert!(!state
            .current_steps
            .get("B")
            .unwrap()
            .tasks_completed
            .contains("B_2"));

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
