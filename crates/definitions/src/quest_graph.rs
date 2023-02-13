use std::collections::{HashMap, HashSet};

use super::quests::*;
use daggy::{
    self,
    petgraph::{
        dot::{Config, Dot},
        graph::IndexType,
    },
    Dag, NodeIndex, Walker,
};

pub struct QuestGraph {
    graph: Dag<String, u32>,
    content: HashMap<StepID, Tasks>,
}

impl QuestGraph {
    pub fn from_quest(quest: Quest) -> Self {
        Self {
            graph: build_graph_from_quest_definition(&quest),
            content: build_content_from_quest_definition(&quest),
        }
    }

    /// Returns the step after `from`
    pub fn next(&self, from: &str) -> Option<Vec<String>> {
        let index = self.get_node_index_by_step_name(from);
        if let Some(index) = index {
            let next_steps = self
                .graph
                .children(index)
                .iter(&self.graph)
                .map(|s| self.graph.node_weight(s.1).unwrap().to_owned())
                .collect::<Vec<String>>();

            Some(next_steps)
        } else {
            None
        }
    }

    /// Returns the step before `from`
    pub fn prev(&self, from: &str) -> Option<Vec<String>> {
        let index = self.get_node_index_by_step_name(from);
        if let Some(index) = index {
            let next_steps = self
                .graph
                .parents(index)
                .iter(&self.graph)
                .map(|s| self.graph.node_weight(s.1).unwrap().to_owned())
                .collect::<Vec<String>>();

            Some(next_steps)
        } else {
            None
        }
    }

    /// Returns steps required for the end of the quests. It returns the steps that directly point to the END, not all the path
    pub fn required_for_end(&self) -> Option<Vec<StepID>> {
        self.prev(END_STEP_ID)
    }

    /// Returns the initial state of the Quest as it's not initialized
    pub fn initial_state(&self) -> QuestState {
        QuestState {
            steps_left: (self.graph.node_count() as u32 - 2), // - 2 becsase START_STEP_ID and END_STEP_ID are also nodes
            required_steps: vec![],                           // All steps at this point
            steps_completed: HashSet::default(),
            next_possible_steps: HashMap::default(),
        }
    }

    fn get_node_index_by_step_name(&self, step: &str) -> Option<NodeIndex> {
        self.graph.graph().node_indices().find(|idx| {
            let item = self.graph.node_weight(*idx);
            if let Some(weight) = item {
                return weight.as_str() == step;
            } else {
                false
            }
        })
    }

    pub fn get_quest_draw(&self) -> Dot<&Dag<String, u32>> {
        Dot::with_config(&self.graph, &[Config::EdgeNoLabel])
    }

    pub fn apply_event(&self, state: QuestState, event: Event) -> Option<QuestState> {
        // if next_possible_steps.contains(&END_STEP_ID.to_string()) && next_possible_steps.len() == 1
        // {
        //     let reboot_state = self.initial_state();
        //     state = QuestState {
        //         steps_left: state.steps_left,
        //         ..reboot_state
        //     };
        //     next_possible_steps = self.next(&state.step_id).unwrap_or_default();
        // }

        let mut cloned_possible = state.next_possible_steps.clone();
        let mut steps_completed = state.steps_completed.clone();

        for (step_id, step_content) in state.next_possible_steps {
            match &step_content.todos {
                Tasks::Single { action_items } => {
                    let mut action_items_cloned = action_items.clone();
                    let matched_action_index = action_items
                        .iter()
                        .position(|action| matches_action((action.clone(), event.action.clone())))
                        .unwrap();

                    action_items_cloned.remove(matched_action_index);

                    if action_items_cloned.is_empty() {
                        cloned_possible.remove(&step_id);
                        let next_current_step_possible_steps =
                            self.next(&step_id).unwrap_or_default();
                        next_current_step_possible_steps.iter().for_each(|step_id| {
                            let step_content = StepContent {
                                todos: self.content.get(step_id).unwrap().clone(),
                                current_subtask: None,
                                subtasks: None,
                            };
                            cloned_possible.insert(step_id.clone(), step_content);
                        });
                        steps_completed.insert(step_id.clone());
                    } else {
                        let step_content = cloned_possible.entry(step_id);
                        step_content.and_modify(|e| {
                            e.todos = Tasks::Single {
                                action_items: action_items_cloned,
                            }
                        });
                    }
                }
                Tasks::Multiple(subtasks) => {
                    if let Some(task_id) = step_content.current_subtask {
                        if let Some(subtask) = subtasks
                            .iter_mut()
                            .find(|subtask| subtask.id == task_id.clone())
                        {
                            let subtask_action = subtask.action_items.remove(0);
                            if matches_action((subtask_action, event.action.clone())) {
                                if subtask.action_items.is_empty() {
                                    subtasks.remove(0);
                                    if let Some(new_current_subtask) = subtasks.get(0) {
                                        // step.current_subtask = Some(new_current_subtask.id.clone());
                                        return Some(QuestState {
                                            next_possible_steps: state.next_possible_steps,
                                            steps_left: state.steps_left,
                                            required_steps: self
                                                .required_for_end()
                                                .unwrap_or_default(),
                                            steps_completed: HashSet::default(),
                                        });
                                    } else {
                                        return Some(QuestState {
                                            next_possible_steps: HashMap::default(),
                                            steps_left: state.steps_left - 1,
                                            required_steps: self
                                                .required_for_end()
                                                .unwrap_or_default(),
                                            steps_completed: HashSet::default(),
                                        });
                                    }
                                } else {
                                    return Some(QuestState {
                                        next_possible_steps: state.next_possible_steps,
                                        steps_left: state.steps_left,
                                        required_steps: self.required_for_end().unwrap_or_default(),
                                        steps_completed: HashSet::default(),
                                    });
                                }
                            }
                        }
                    } else if let Some(task) = subtasks.get_mut(0) {
                        let subtask_action = task.action_items.remove(0);
                        if matches_action((subtask_action, event.action.clone())) {
                            // step.current_subtask = Some(task.id.clone());
                            return Some(QuestState {
                                next_possible_steps: state.next_possible_steps,
                                steps_left: state.steps_left,
                                required_steps: vec![],
                                steps_completed: HashSet::default(),
                            });
                        }
                    }
                }
                Tasks::None => {} // TODO: Should be removed asap
            };
        }
        None
    }
}

fn build_graph_from_quest_definition(quest: &Quest) -> Dag<String, u32> {
    let mut dag = daggy::Dag::<String, u32, u32>::new();
    let starting_step = Step {
        id: START_STEP_ID.to_string(),
        description: "COMMON START NODE".to_string(),
        tasks: Tasks::None,
        on_complete_hook: None,
    };
    let ending_step = Step {
        id: END_STEP_ID.to_string(),
        description: "COMMON END NODE".to_string(),
        tasks: Tasks::None,
        on_complete_hook: None,
    };

    let start_node = dag.add_node(starting_step.id);
    let end_node = dag.add_node(ending_step.id);
    let mut nodes: HashMap<String, NodeIndex> = HashMap::new();

    for (step_from, step_to) in &quest.definition.connections {
        // Validate if steps are in defined in the quest
        if quest.contanins_step(step_from) && quest.contanins_step(step_to) {
            if let Some(node_from) = nodes.get(step_from) {
                let (_, node_to) =
                    dag.add_child(*node_from, node_from.index() as u32, step_to.clone());
                nodes.insert(step_to.clone(), node_to);
            } else {
                let node_from = dag.add_node(step_from.clone());
                nodes.insert(step_from.clone(), node_from);
                let (_, node_to) =
                    dag.add_child(node_from, node_from.index() as u32, step_to.clone());
                nodes.insert(step_to.clone(), node_to);
            }
        }
    }

    let steps_without_to = quest.get_steps_without_to();
    for step in steps_without_to {
        if let Some(node_index) = nodes.get(&step) {
            dag.add_edge(*node_index, end_node, 0).unwrap();
        }
    }

    let steps_without_from = quest.get_steps_without_from();
    for step in steps_without_from {
        if let Some(node_index) = nodes.get(&step) {
            dag.add_edge(start_node, *node_index, 0).unwrap();
        }
    }

    dag
}

fn build_content_from_quest_definition(quest: &Quest) -> HashMap<StepID, Tasks> {
    let mut content_map = HashMap::new();
    for step in &quest.definition.steps {
        let content = step.tasks.clone();
        content_map.insert(step.id.clone(), content);
    }
    content_map
}

fn matches_action((action, event_action): (Action, Action)) -> bool {
    action == event_action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_quest_graph_properly() {
        let quest = Quest {
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: QuestDefinition {
                connections: vec![
                    ("A".to_string(), "B".to_string()),
                    ("B".to_string(), "C".to_string()),
                    ("C".to_string(), "D".to_string()),
                ],
                steps: vec![
                    Step {
                        id: "A".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                ],
            },
        };

        let graph = QuestGraph::from_quest(quest);

        let next = graph.next(START_STEP_ID).unwrap();
        assert_eq!(next.len(), 1);
        assert_eq!(next[0], "A");
        let next = graph.next("A").unwrap();
        assert_eq!(next.len(), 1);
        assert_eq!(next[0], "B");
        let next = graph.next("B").unwrap();
        assert_eq!(next.len(), 1);
        assert_eq!(next[0], "C");
        let next = graph.next("C").unwrap();
        assert_eq!(next.len(), 1);
        assert_eq!(next[0], "D");
    }

    #[test]
    fn build_quest_graph_with_multiple_starting_points_properly() {
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
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "A2".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                ],
            },
        };

        let graph = QuestGraph::from_quest(quest);
        let next = graph.next(START_STEP_ID).unwrap();
        assert_eq!(next.len(), 2);
        assert!(next.contains(&"A1".to_string()));
        assert!(next.contains(&"A2".to_string()));

        // A1 Path
        let next = graph.next("A1").unwrap();
        assert_eq!(next.len(), 1);
        assert_eq!(next[0], "B");
        let next = graph.next("B").unwrap();
        assert_eq!(next.len(), 1);
        assert_eq!(next[0], "C");
        let next = graph.next("C").unwrap();
        assert_eq!(next.len(), 1);
        assert_eq!(next[0], END_STEP_ID);

        // A2 Path
        let next = graph.next("A2").unwrap();
        assert_eq!(next.len(), 1);
        assert_eq!(next[0], "D");
        let next = graph.next("D").unwrap();
        assert_eq!(next.len(), 1);
        assert_eq!(next[0], END_STEP_ID);
    }

    #[test]
    fn build_quest_graph_with_multiple_pivot_points() {
        let quest = Quest {
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: QuestDefinition {
                connections: vec![
                    ("A".to_string(), "B1".to_string()),
                    ("A".to_string(), "B2".to_string()),
                    ("A".to_string(), "B3".to_string()),
                    ("B1".to_string(), "C".to_string()),
                    ("C".to_string(), "D".to_string()),
                ],
                steps: vec![
                    Step {
                        id: "A".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B1".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B2".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B3".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                ],
            },
        };

        let graph = QuestGraph::from_quest(quest);
        let next = graph.next(START_STEP_ID).unwrap();
        assert_eq!(next, vec!["A"]);
        let next = graph.next("A").unwrap();
        assert_eq!(next.len(), 3);
        assert!(next.contains(&"B1".to_string()));
        assert!(next.contains(&"B2".to_string()));
        assert!(next.contains(&"B3".to_string()));

        // Path B1
        let next = graph.next("B1").unwrap();
        assert_eq!(next, vec!["C"]);
        let next = graph.next("C").unwrap();
        assert_eq!(next, vec!["D"]);
        let next = graph.next("D").unwrap();
        assert_eq!(next, vec![END_STEP_ID]);
        // Path B2
        let next = graph.next("B2").unwrap();
        assert_eq!(next, vec![END_STEP_ID]);
        // Path B3
        let next = graph.next("B3").unwrap();
        assert_eq!(next, vec![END_STEP_ID]);
    }

    #[test]
    fn quest_graph_prev_works_properly() {
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
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "A2".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                ],
            },
        };

        let graph = QuestGraph::from_quest(quest);

        let prev_step = graph.prev("A1").unwrap();
        assert_eq!(prev_step, vec![START_STEP_ID]);
        let prev_step = graph.prev("B").unwrap();
        assert_eq!(prev_step, vec!["A1"]);
        let prev_step = graph.prev("D").unwrap();
        assert_eq!(prev_step, vec!["A2"])
    }

    #[test]
    fn quest_graph_steps_required_for_end() {
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
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "A2".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                        },
                        on_complete_hook: None,
                    },
                ],
            },
        };

        assert!(quest.is_valid().is_ok());
        let graph = QuestGraph::from_quest(quest);
        let steps_required_for_end = graph.required_for_end().unwrap();
        assert!(steps_required_for_end.contains(&"D".to_string()));
        assert!(steps_required_for_end.contains(&"C".to_string()));
    }

    #[test]
    fn matches_action_works() {
        let result = matches_action((
            Action::Location {
                coordinates: Coordinates(10, 10),
            },
            Action::Location {
                coordinates: Coordinates(10, 10),
            },
        ));
        assert!(result);

        let result = matches_action((
            Action::Location {
                coordinates: Coordinates(10, 10),
            },
            Action::Location {
                coordinates: Coordinates(10, 20),
            },
        ));
        assert!(!result);

        let result = matches_action((
            Action::Location {
                coordinates: Coordinates(10, 10),
            },
            Action::Jump {
                coordinates: Coordinates(10, 10),
            },
        ));
        assert!(!result);
    }

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

        let mut quest_graph = QuestGraph::from_quest(quest);
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
        let mut state = quest_graph.initial_state();
        state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        // assert_eq!(state.step_id, START_STEP_ID);
        // println!("{state:?}");
        // // assert!(state.next_possible_steps.contains(&"A1".to_string()));
        // // assert!(state.next_possible_steps.contains(&"A2".to_string()));
        // assert_eq!(state.steps_left, 5);
        // state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        // assert_eq!(state.step_id, "A1");
        // println!("{state:?}");
        // // assert!(state.next_possible_steps.contains(&"B".to_string()));
        // assert_eq!(state.steps_left, 4);
        // state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        // assert_eq!(state.step_id, "B");
        // println!("{state:?}");
        // // assert!(state.next_possible_steps.contains(&"C".to_string()));
        // assert_eq!(state.steps_left, 3);
        // state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        // assert_eq!(state.step_id, "C");
        // println!("{state:?}");
        // // assert!(state.next_possible_steps.contains(&END_STEP_ID.to_string()));
        // assert_eq!(state.steps_left, 2);
        // state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        // assert_eq!(state.step_id, "A2");
        // println!("{state:?}");
        // // assert!(state.next_possible_steps.contains(&"D".to_string()));
        // assert_eq!(state.steps_left, 1);
        // state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        // assert_eq!(state.step_id, "D");
        // println!("{state:?}");
        // // assert!(state.next_possible_steps.contains(&END_STEP_ID.to_string()));
        // assert_eq!(state.steps_left, 0);
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

        let mut quest_graph = QuestGraph::from_quest(quest);
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
        let mut state = quest_graph.initial_state();
        state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        println!("{state:?}");
        state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        println!("{state:?}");
        state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        println!("{state:?}");
        state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        println!("{state:?}");
        state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        println!("{state:?}");
        state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        println!("{state:?}");
        state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        println!("{state:?}");
        state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        println!("{state:?}");
        state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        println!("{state:?}");
        // state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        // assert_eq!(state.current_step_id, "A1");
        // assert_eq!(state.next_current_subtask, None);
        // assert!(state.next_possible_steps.contains(&"B".to_string()));
        // assert_eq!(state.steps_left, 4);
        // state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        // assert_eq!(state.current_step_id, "B");
        // assert_eq!(state.next_current_subtask, None);
        // assert!(state.next_possible_steps.contains(&"C".to_string()));
        // assert_eq!(state.steps_left, 3);
        // state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        // assert_eq!(state.current_step_id, "C");
        // assert_eq!(state.next_current_subtask, None);
        // assert!(state.next_possible_steps.contains(&END_STEP_ID.to_string()));
        // assert_eq!(state.steps_left, 2);
        // state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        // assert_eq!(state.current_step_id, "A2");
        // assert_eq!(state.next_current_subtask, None);
        // assert!(state.next_possible_steps.contains(&"D".to_string()));
        // assert_eq!(state.steps_left, 1);
        // state = quest_graph.apply_event(state, events.remove(0)).unwrap();
        // assert_eq!(state.current_step_id, "D");
        // assert_eq!(state.next_current_subtask, None);
        // assert!(state.next_possible_steps.contains(&END_STEP_ID.to_string()));
        // assert_eq!(state.steps_left, 0);
    }
}
