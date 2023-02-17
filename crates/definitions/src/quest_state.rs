use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    quest_graph::{matches_action, QuestGraph},
    quests::{Event, Quest, StepID, SubTask, Tasks, END_STEP_ID, START_STEP_ID},
};

#[derive(Serialize, Deserialize)]
pub struct QuestUpdate {
    pub state: QuestState,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct QuestState {
    /// Next possible steps
    pub next_possible_steps: HashMap<StepID, StepContent>,
    /// Steps left to complete the Quest
    pub steps_left: u32,
    /// Required Steps for END
    pub required_steps: Vec<StepID>,
    /// Quest Steps Completed
    pub steps_completed: HashSet<StepID>,
    /// Subtasks completed. Inner Step tasks
    ///
    /// String in key refers to SubTask ID
    ///
    pub subtasks_completed: Option<HashSet<String>>,
}

impl QuestState {
    pub fn is_completed(&self) -> bool {
        self.required_steps
            .iter()
            .all(|step| self.steps_completed.contains(step))
    }

    pub fn apply_event(&self, quest_graph: &QuestGraph, event: &Event) -> QuestState {
        let state = self.clone();
        // We do many clones because we don't want to mutate the state given directly. And also, we don't want to keep a state in the QuestGraph
        // We clone the next possible steps in order to mutate this instance for the new state
        let mut next_possible_steps_cloned = state.next_possible_steps.clone();
        // We clone the current completed steps in order to add the new ones for the event given
        let mut steps_completed = state.steps_completed.clone();
        // We move the current completed subtasks in order to add new ones for the event given
        let mut quest_subtasks_completed = state.subtasks_completed;

        for (step_id, step_content) in state.next_possible_steps {
            match &step_content.todos {
                Tasks::Single { action_items } => {
                    let mut action_items_cloned = action_items.clone();
                    match action_items
                        .iter()
                        .position(|action| matches_action((action.clone(), event.action.clone())))
                    {
                        Some(matched_action_index) => {
                            action_items_cloned.remove(matched_action_index);
                            if action_items_cloned.is_empty() {
                                next_possible_steps_cloned.remove(&step_id);
                                let next_current_step_possible_steps =
                                    quest_graph.next(&step_id).unwrap_or_default();
                                next_current_step_possible_steps.iter().for_each(|step_id| {
                                    if step_id != END_STEP_ID {
                                        let step_content = StepContent {
                                            todos: quest_graph
                                                .tasks_by_step
                                                .get(step_id)
                                                .unwrap()
                                                .clone(),
                                        };
                                        next_possible_steps_cloned
                                            .insert(step_id.clone(), step_content);
                                    }
                                });
                                steps_completed.insert(step_id.clone());
                            } else {
                                let step_content = next_possible_steps_cloned.entry(step_id);
                                step_content.and_modify(|e| {
                                    e.todos = Tasks::Single {
                                        action_items: action_items_cloned,
                                    }
                                });
                            }
                        }
                        None => continue,
                    }
                }
                Tasks::Multiple(subtasks) => {
                    let mut subtasks_cloned = subtasks.clone();
                    for (i, subtask) in subtasks.iter().enumerate() {
                        let mut actions_items_cloned = subtask.action_items.clone();
                        match subtask.action_items.iter().position(|action| {
                            matches_action((action.clone(), event.action.clone()))
                        }) {
                            Some(matched_action_index) => {
                                actions_items_cloned.remove(matched_action_index);

                                if actions_items_cloned.is_empty() {
                                    if let Some(current_subtasks_completed) =
                                        &mut quest_subtasks_completed
                                    {
                                        current_subtasks_completed.insert(subtask.id.clone());
                                    } else {
                                        let mut subtasks = HashSet::new();
                                        subtasks.insert(subtask.id.clone());
                                        quest_subtasks_completed = Some(subtasks)
                                    }
                                    subtasks_cloned.remove(i);
                                } else {
                                    subtasks_cloned[i] = SubTask {
                                        id: subtask.id.clone(),
                                        description: subtask.description.clone(),
                                        action_items: actions_items_cloned,
                                    };
                                }
                            }
                            None => continue,
                        }
                    }
                    if subtasks_cloned.is_empty() {
                        next_possible_steps_cloned.remove(&step_id);
                        let next_current_step_possible_steps =
                            quest_graph.next(&step_id).unwrap_or_default();
                        next_current_step_possible_steps.iter().for_each(|step_id| {
                            if step_id != END_STEP_ID {
                                let step_content = StepContent {
                                    todos: quest_graph.tasks_by_step.get(step_id).unwrap().clone(),
                                };
                                next_possible_steps_cloned.insert(step_id.clone(), step_content);
                            }
                        });
                        steps_completed.insert(step_id.clone());
                    } else {
                        let step = next_possible_steps_cloned.get_mut(&step_id).unwrap();
                        step.todos = Tasks::Multiple(subtasks_cloned)
                    }
                }
                // We use this type for the START and END nodes because we consider them as "Step"
                Tasks::None => {}
            };
        }

        QuestState {
            next_possible_steps: next_possible_steps_cloned,
            steps_left: (quest_graph.total_steps() - steps_completed.len()) as u32,
            required_steps: state.required_steps,
            steps_completed,
            subtasks_completed: quest_subtasks_completed,
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
                        todos: value.tasks_by_step.get(step).unwrap().clone(),
                    },
                )
            })
            .collect::<HashMap<String, StepContent>>();

        Self {
            next_possible_steps,
            required_steps: value.required_for_end().unwrap_or_default(),
            steps_left: value.total_steps() as u32,
            steps_completed: HashSet::default(),
            subtasks_completed: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct StepContent {
    pub todos: Tasks,
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
                        tasks: Tasks::Single {
                            action_items: vec![
                                Action::Location {
                                    coordinates: Coordinates(10, 10),
                                },
                                Action::Jump {
                                    coordinates: Coordinates(10, 11),
                                },
                            ],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "A2".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::NPCInteraction {
                                npc_id: "NPC_IDEN".to_string(),
                            }],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::Jump {
                                coordinates: Coordinates(20, 10),
                            }],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::Jump {
                                coordinates: Coordinates(20, 20),
                            }],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::NPCInteraction {
                                npc_id: "OTHER_NPC".to_string(),
                            }],
                        },
                        on_complete_hook: None,
                    },
                ],
            },
        };
        let quest_graph = QuestGraph::from(&quest);
        let mut events = vec![
            Event {
                address: "0xA".to_string(),
                timestamp: 111111,
                action: Action::Location {
                    coordinates: Coordinates(10, 10),
                },
            },
            Event {
                address: "0xA".to_string(),
                timestamp: 111115,
                action: Action::Jump {
                    coordinates: Coordinates(10, 11),
                },
            },
            Event {
                address: "0xA".to_string(),
                timestamp: 111118,
                action: Action::Jump {
                    coordinates: Coordinates(20, 10),
                },
            },
            Event {
                address: "0xA".to_string(),
                timestamp: 111118,
                action: Action::Jump {
                    coordinates: Coordinates(20, 20),
                },
            },
            Event {
                address: "0xA".to_string(),
                timestamp: 111118,
                action: Action::NPCInteraction {
                    npc_id: "NPC_IDEN".to_string(),
                },
            },
            Event {
                address: "0xA".to_string(),
                timestamp: 111118,
                action: Action::NPCInteraction {
                    npc_id: "OTHER_NPC".to_string(),
                },
            },
        ];
        let mut state = QuestState::from(&quest_graph);
        assert!(state.next_possible_steps.contains_key("A1")); // branch 1
        assert!(state.next_possible_steps.contains_key("A2")); // branch 2
        assert_eq!(state.next_possible_steps.len(), 2);
        assert!(state.steps_completed.is_empty());
        assert_eq!(state.steps_left, 5);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));
        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.next_possible_steps.contains_key("A1"));
        assert!(state.next_possible_steps.contains_key("A2"));
        assert_eq!(state.next_possible_steps.len(), 2);
        assert!(state.steps_completed.is_empty());
        assert_eq!(state.steps_left, 5);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));
        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.next_possible_steps.contains_key("B"));
        assert!(state.next_possible_steps.contains_key("A2"));
        assert_eq!(state.next_possible_steps.len(), 2);
        assert!(state.steps_completed.contains(&"A1".to_string()));
        assert_eq!(state.steps_left, 4);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));
        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.next_possible_steps.contains_key("C"));
        assert!(state.next_possible_steps.contains_key("A2"));
        assert_eq!(state.next_possible_steps.len(), 2);
        assert!(state.steps_completed.contains(&"A1".to_string()));
        assert!(state.steps_completed.contains(&"B".to_string()));
        assert_eq!(state.steps_left, 3);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));
        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.next_possible_steps.contains_key("A2"));
        assert_eq!(state.next_possible_steps.len(), 1);
        assert!(state.steps_completed.contains(&"A1".to_string()));
        assert!(state.steps_completed.contains(&"B".to_string()));
        assert!(state.steps_completed.contains(&"C".to_string()));
        assert_eq!(state.steps_left, 2);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));
        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.next_possible_steps.contains_key("D"));
        assert_eq!(state.next_possible_steps.len(), 1);
        assert!(state.steps_completed.contains(&"A1".to_string()));
        assert!(state.steps_completed.contains(&"A2".to_string()));
        assert!(state.steps_completed.contains(&"B".to_string()));
        assert!(state.steps_completed.contains(&"C".to_string()));
        assert_eq!(state.steps_left, 1);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));
        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.next_possible_steps.is_empty());
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
                        tasks: Tasks::Multiple(vec![
                            SubTask {
                                id: "A_1".to_string(),
                                description: "".to_string(),
                                action_items: vec![
                                    Action::Jump {
                                        coordinates: Coordinates(10, 10),
                                    },
                                    Action::Location {
                                        coordinates: Coordinates(15, 10),
                                    },
                                ],
                            },
                            SubTask {
                                id: "A_2".to_string(),
                                description: "".to_string(),
                                action_items: vec![
                                    Action::NPCInteraction {
                                        npc_id: "NPC_ID".to_string(),
                                    },
                                    Action::Location {
                                        coordinates: Coordinates(15, 14),
                                    },
                                ],
                            },
                        ]),
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Multiple(vec![
                            SubTask {
                                id: "B_1".to_string(),
                                description: "".to_string(),
                                action_items: vec![
                                    Action::Jump {
                                        coordinates: Coordinates(10, 20),
                                    },
                                    Action::Location {
                                        coordinates: Coordinates(23, 14),
                                    },
                                ],
                            },
                            SubTask {
                                id: "B_2".to_string(),
                                description: "".to_string(),
                                action_items: vec![
                                    Action::Custom {
                                        id: "a".to_string(),
                                    },
                                    Action::Location {
                                        coordinates: Coordinates(40, 10),
                                    },
                                ],
                            },
                        ]),
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::Jump {
                                coordinates: Coordinates(20, 20),
                            }],
                        },
                        on_complete_hook: None,
                    },
                ],
            },
        };

        let quest_graph = QuestGraph::from(&quest);
        let mut events = vec![
            Event {
                address: "0xA".to_string(),
                timestamp: 111111,
                action: Action::Jump {
                    coordinates: Coordinates(10, 10),
                },
            },
            Event {
                address: "0xA".to_string(),
                timestamp: 111115,
                action: Action::Location {
                    coordinates: Coordinates(15, 10),
                },
            },
            Event {
                address: "0xA".to_string(),
                timestamp: 111118,
                action: Action::NPCInteraction {
                    npc_id: "NPC_ID".to_string(),
                },
            },
            Event {
                address: "0xA".to_string(),
                timestamp: 111115,
                action: Action::Location {
                    coordinates: Coordinates(15, 14),
                },
            },
            Event {
                address: "0xA".to_string(),
                timestamp: 111118,
                action: Action::Jump {
                    coordinates: Coordinates(10, 20),
                },
            },
            Event {
                address: "0xA".to_string(),
                timestamp: 111118,
                action: Action::Location {
                    coordinates: Coordinates(23, 14),
                },
            },
            Event {
                address: "0xA".to_string(),
                timestamp: 111118,
                action: Action::Custom {
                    id: "a".to_string(),
                },
            },
            Event {
                address: "0xA".to_string(),
                timestamp: 111118,
                action: Action::Location {
                    coordinates: Coordinates(40, 10),
                },
            },
            Event {
                address: "0xA".to_string(),
                timestamp: 111118,
                action: Action::Jump {
                    coordinates: Coordinates(20, 20),
                },
            },
        ];
        let mut state = QuestState::from(&quest_graph);

        assert!(state.next_possible_steps.contains_key("A"));
        if let Tasks::Multiple(subtasks) = &state.next_possible_steps.get("A").unwrap().todos {
            assert_eq!(subtasks.len(), 2);
            assert_eq!(
                subtasks.get(0).unwrap(),
                &SubTask {
                    id: "A_1".to_string(),
                    description: "".to_string(),
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
                subtasks.get(1).unwrap(),
                &SubTask {
                    id: "A_2".to_string(),
                    description: "".to_string(),
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
        } else {
            panic!()
        }
        assert_eq!(state.next_possible_steps.len(), 1);
        assert!(state.steps_completed.is_empty());
        assert_eq!(state.steps_left, 3);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert_eq!(state.required_steps.len(), 1);
        assert!(state.subtasks_completed.is_none());

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.next_possible_steps.contains_key("A"));
        assert_eq!(state.next_possible_steps.len(), 1);
        if let Tasks::Multiple(subtasks) = &state.next_possible_steps.get("A").unwrap().todos {
            assert_eq!(subtasks.len(), 2);
            assert_eq!(
                subtasks.get(0).unwrap(),
                &SubTask {
                    id: "A_1".to_string(),
                    description: "".to_string(),
                    action_items: vec![Action::Location {
                        coordinates: Coordinates(15, 10),
                    },]
                }
            );
            assert_eq!(
                subtasks.get(1).unwrap(),
                &SubTask {
                    id: "A_2".to_string(),
                    description: "".to_string(),
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
        } else {
            panic!()
        }
        assert!(state.steps_completed.is_empty());
        assert_eq!(state.steps_left, 3);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert_eq!(state.required_steps.len(), 1);
        assert!(state.subtasks_completed.is_none());

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.next_possible_steps.contains_key("A"));
        assert_eq!(state.next_possible_steps.len(), 1);
        if let Tasks::Multiple(subtasks) = &state.next_possible_steps.get("A").unwrap().todos {
            assert_eq!(subtasks.len(), 1);
            assert_eq!(
                subtasks.get(0).unwrap(),
                &SubTask {
                    id: "A_2".to_string(),
                    description: "".to_string(),
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
        } else {
            panic!()
        }
        assert!(state.steps_completed.is_empty());
        assert_eq!(state.steps_left, 3);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert_eq!(state.required_steps.len(), 1);
        assert!(state.subtasks_completed.is_some());
        assert!(state.subtasks_completed.as_ref().unwrap().contains("A_1"));
        assert_eq!(state.subtasks_completed.as_ref().unwrap().len(), 1);
        state = state.apply_event(&quest_graph, &events.remove(0));

        assert!(state.next_possible_steps.contains_key("A"));
        assert_eq!(state.next_possible_steps.len(), 1);
        if let Tasks::Multiple(subtasks) = &state.next_possible_steps.get("A").unwrap().todos {
            assert_eq!(subtasks.len(), 1);
            assert_eq!(
                subtasks.get(0).unwrap(),
                &SubTask {
                    id: "A_2".to_string(),
                    description: "".to_string(),
                    action_items: vec![Action::Location {
                        coordinates: Coordinates(15, 14),
                    },]
                }
            );
        } else {
            panic!()
        }
        assert!(state.steps_completed.is_empty());
        assert_eq!(state.steps_left, 3);
        assert_eq!(state.required_steps.len(), 1);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.subtasks_completed.is_some());
        assert!(state.subtasks_completed.as_ref().unwrap().contains("A_1"));
        assert_eq!(state.subtasks_completed.as_ref().unwrap().len(), 1);
        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.next_possible_steps.contains_key("B"));
        assert_eq!(state.next_possible_steps.len(), 1);
        if let Tasks::Multiple(subtasks) = &state.next_possible_steps.get("B").unwrap().todos {
            assert_eq!(subtasks.len(), 2);
            assert_eq!(
                subtasks.get(0).unwrap(),
                &SubTask {
                    id: "B_1".to_string(),
                    description: "".to_string(),
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
                subtasks.get(1).unwrap(),
                &SubTask {
                    id: "B_2".to_string(),
                    description: "".to_string(),
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
        } else {
            panic!()
        }
        assert!(state.steps_completed.contains(&"A".to_string()));
        assert_eq!(state.steps_left, 2);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert_eq!(state.required_steps.len(), 1);
        assert!(state.subtasks_completed.is_some());
        assert!(state.subtasks_completed.as_ref().unwrap().contains("A_1"));
        assert!(state.subtasks_completed.as_ref().unwrap().contains("A_2"));
        assert_eq!(state.subtasks_completed.as_ref().unwrap().len(), 2);

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.next_possible_steps.contains_key("B"));
        assert_eq!(state.next_possible_steps.len(), 1);
        if let Tasks::Multiple(subtasks) = &state.next_possible_steps.get("B").unwrap().todos {
            assert_eq!(subtasks.len(), 2);
            assert_eq!(
                subtasks.get(0).unwrap(),
                &SubTask {
                    id: "B_1".to_string(),
                    description: "".to_string(),
                    action_items: vec![Action::Location {
                        coordinates: Coordinates(23, 14),
                    },],
                },
            );
            assert_eq!(
                subtasks.get(1).unwrap(),
                &SubTask {
                    id: "B_2".to_string(),
                    description: "".to_string(),
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
        } else {
            panic!()
        }
        assert!(state.steps_completed.contains(&"A".to_string()));
        assert_eq!(state.steps_left, 2);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.subtasks_completed.is_some());
        assert!(state.subtasks_completed.as_ref().unwrap().contains("A_1"));
        assert!(state.subtasks_completed.as_ref().unwrap().contains("A_2"));
        assert_eq!(state.subtasks_completed.as_ref().unwrap().len(), 2);

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.next_possible_steps.contains_key("B"));
        assert_eq!(state.next_possible_steps.len(), 1);
        if let Tasks::Multiple(subtasks) = &state.next_possible_steps.get("B").unwrap().todos {
            assert_eq!(subtasks.len(), 1);
            assert_eq!(
                subtasks.get(0).unwrap(),
                &SubTask {
                    id: "B_2".to_string(),
                    description: "".to_string(),
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
        } else {
            panic!()
        }
        assert!(state.steps_completed.contains(&"A".to_string()));
        assert_eq!(state.steps_left, 2);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert_eq!(state.required_steps.len(), 1);
        assert!(state.subtasks_completed.is_some());
        assert!(state.subtasks_completed.as_ref().unwrap().contains("A_1"));
        assert!(state.subtasks_completed.as_ref().unwrap().contains("A_2"));
        assert!(state.subtasks_completed.as_ref().unwrap().contains("B_1"));
        assert_eq!(state.subtasks_completed.as_ref().unwrap().len(), 3);
        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.next_possible_steps.contains_key("B"));
        assert_eq!(state.next_possible_steps.len(), 1);
        if let Tasks::Multiple(subtasks) = &state.next_possible_steps.get("B").unwrap().todos {
            assert_eq!(subtasks.len(), 1);
            assert_eq!(
                subtasks.get(0).unwrap(),
                &SubTask {
                    id: "B_2".to_string(),
                    description: "".to_string(),
                    action_items: vec![Action::Location {
                        coordinates: Coordinates(40, 10),
                    },],
                },
            );
        } else {
            panic!()
        }
        assert!(state.steps_completed.contains(&"A".to_string()));
        assert_eq!(state.steps_left, 2);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert_eq!(state.required_steps.len(), 1);
        assert!(state.subtasks_completed.is_some());
        assert!(state.subtasks_completed.as_ref().unwrap().contains("A_1"));
        assert!(state.subtasks_completed.as_ref().unwrap().contains("A_2"));
        assert!(state.subtasks_completed.as_ref().unwrap().contains("B_1"));
        assert_eq!(state.subtasks_completed.as_ref().unwrap().len(), 3);

        state = state.apply_event(&quest_graph, &events.remove(0));
        assert!(state.next_possible_steps.contains_key("C"));
        assert_eq!(state.next_possible_steps.len(), 1);
        assert!(state.steps_completed.contains(&"A".to_string()));
        assert!(state.steps_completed.contains(&"B".to_string()));
        assert_eq!(state.steps_left, 1);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert_eq!(state.required_steps.len(), 1);
        assert!(state.subtasks_completed.is_some());
        assert!(state.subtasks_completed.as_ref().unwrap().contains("A_1"));
        assert!(state.subtasks_completed.as_ref().unwrap().contains("A_2"));
        assert!(state.subtasks_completed.as_ref().unwrap().contains("B_1"));
        assert!(state.subtasks_completed.as_ref().unwrap().contains("B_2"));
        assert_eq!(state.subtasks_completed.as_ref().unwrap().len(), 4);
        state = state.apply_event(&quest_graph, &events.remove(0));

        assert_eq!(state.next_possible_steps.len(), 0);
        assert!(state.steps_completed.contains(&"A".to_string()));
        assert!(state.steps_completed.contains(&"B".to_string()));
        assert!(state.steps_completed.contains(&"C".to_string()));
        assert_eq!(state.steps_left, 0);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert_eq!(state.required_steps.len(), 1);
        assert!(state.subtasks_completed.is_some());
        assert!(state.subtasks_completed.as_ref().unwrap().contains("A_1"));
        assert!(state.subtasks_completed.as_ref().unwrap().contains("A_2"));
        assert!(state.subtasks_completed.as_ref().unwrap().contains("B_1"));
        assert!(state.subtasks_completed.as_ref().unwrap().contains("B_2"));
        assert_eq!(state.subtasks_completed.as_ref().unwrap().len(), 4);
        assert!(state.is_completed())
    }
}
