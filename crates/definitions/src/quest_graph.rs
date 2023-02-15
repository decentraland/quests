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
use serde::{Deserialize, Serialize};

pub struct QuestGraph {
    graph: Dag<String, u32>,
    tasks_by_step: HashMap<StepID, Tasks>,
}

impl QuestGraph {
    pub fn from_quest(quest: &Quest) -> Self {
        Self {
            graph: build_graph_from_quest_definition(quest),
            tasks_by_step: build_tasks_by_step_from_quest_definition(quest),
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

    pub fn total_steps(&self) -> usize {
        // - 2 becsase START_STEP_ID and END_STEP_ID are also nodes
        self.graph.node_count() - 2
    }

    /// Returns the initial state of the Quest as it's not initialized
    pub fn initial_state(&self) -> QuestState {
        let next_possible_steps = self
            .next(START_STEP_ID)
            .unwrap_or_default()
            .iter()
            .map(|step| {
                (
                    step.clone(),
                    StepContent {
                        todos: self.tasks_by_step.get(step).unwrap().clone(),
                    },
                )
            })
            .collect::<HashMap<String, StepContent>>();

        QuestState {
            next_possible_steps,
            required_steps: self.required_for_end().unwrap_or_default(),
            steps_left: self.total_steps() as u32,
            steps_completed: HashSet::default(),
            subtasks_completed: None,
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

    pub fn apply_event(&self, state: QuestState, event: &Event) -> QuestState {
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
                                    self.next(&step_id).unwrap_or_default();
                                next_current_step_possible_steps.iter().for_each(|step_id| {
                                    if step_id != END_STEP_ID {
                                        let step_content = StepContent {
                                            todos: self.tasks_by_step.get(step_id).unwrap().clone(),
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
                            self.next(&step_id).unwrap_or_default();
                        next_current_step_possible_steps.iter().for_each(|step_id| {
                            if step_id != END_STEP_ID {
                                let step_content = StepContent {
                                    todos: self.tasks_by_step.get(step_id).unwrap().clone(),
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
            steps_left: (self.total_steps() - steps_completed.len()) as u32,
            required_steps: state.required_steps,
            steps_completed,
            subtasks_completed: quest_subtasks_completed,
        }
    }
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
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct StepContent {
    pub todos: Tasks,
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

fn build_tasks_by_step_from_quest_definition(quest: &Quest) -> HashMap<StepID, Tasks> {
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

        let graph = QuestGraph::from_quest(&quest);

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

        let graph = QuestGraph::from_quest(&quest);
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

        let graph = QuestGraph::from_quest(&quest);
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

        let graph = QuestGraph::from_quest(&quest);

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
        let graph = QuestGraph::from_quest(&quest);
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
        let quest_graph = QuestGraph::from_quest(&quest);
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
        assert!(state.next_possible_steps.contains_key("A1")); // branch 1
        assert!(state.next_possible_steps.contains_key("A2")); // branch 2
        assert_eq!(state.next_possible_steps.len(), 2);
        assert!(state.steps_completed.is_empty());
        assert_eq!(state.steps_left, 5);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));
        state = quest_graph.apply_event(state, &events.remove(0));
        assert!(state.next_possible_steps.contains_key("A1"));
        assert!(state.next_possible_steps.contains_key("A2"));
        assert_eq!(state.next_possible_steps.len(), 2);
        assert!(state.steps_completed.is_empty());
        assert_eq!(state.steps_left, 5);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));
        state = quest_graph.apply_event(state, &events.remove(0));
        assert!(state.next_possible_steps.contains_key("B"));
        assert!(state.next_possible_steps.contains_key("A2"));
        assert_eq!(state.next_possible_steps.len(), 2);
        assert!(state.steps_completed.contains(&"A1".to_string()));
        assert_eq!(state.steps_left, 4);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));
        state = quest_graph.apply_event(state, &events.remove(0));
        assert!(state.next_possible_steps.contains_key("C"));
        assert!(state.next_possible_steps.contains_key("A2"));
        assert_eq!(state.next_possible_steps.len(), 2);
        assert!(state.steps_completed.contains(&"A1".to_string()));
        assert!(state.steps_completed.contains(&"B".to_string()));
        assert_eq!(state.steps_left, 3);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));
        state = quest_graph.apply_event(state, &events.remove(0));
        assert!(state.next_possible_steps.contains_key("A2"));
        assert_eq!(state.next_possible_steps.len(), 1);
        assert!(state.steps_completed.contains(&"A1".to_string()));
        assert!(state.steps_completed.contains(&"B".to_string()));
        assert!(state.steps_completed.contains(&"C".to_string()));
        assert_eq!(state.steps_left, 2);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));
        state = quest_graph.apply_event(state, &events.remove(0));
        assert!(state.next_possible_steps.contains_key("D"));
        assert_eq!(state.next_possible_steps.len(), 1);
        assert!(state.steps_completed.contains(&"A1".to_string()));
        assert!(state.steps_completed.contains(&"A2".to_string()));
        assert!(state.steps_completed.contains(&"B".to_string()));
        assert!(state.steps_completed.contains(&"C".to_string()));
        assert_eq!(state.steps_left, 1);
        assert!(state.required_steps.contains(&"C".to_string()));
        assert!(state.required_steps.contains(&"D".to_string()));
        state = quest_graph.apply_event(state, &events.remove(0));
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

        let quest_graph = QuestGraph::from_quest(&quest);
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
        state = quest_graph.apply_event(state, &events.remove(0));
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
        state = quest_graph.apply_event(state, &events.remove(0));
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
        state = quest_graph.apply_event(state, &events.remove(0));
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
        state = quest_graph.apply_event(state, &events.remove(0));
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
        state = quest_graph.apply_event(state, &events.remove(0));
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
        state = quest_graph.apply_event(state, &events.remove(0));
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
        state = quest_graph.apply_event(state, &events.remove(0));
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
        state = quest_graph.apply_event(state, &events.remove(0));
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
        state = quest_graph.apply_event(state, &events.remove(0));
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
