use std::collections::HashMap;

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
}

impl QuestGraph {
    pub fn from_quest(quest: Quest) -> Self {
        let graph = build_graph_from_quest_definition(&quest);
        Self { graph }
    }

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
                            repeat: None,
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
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
                            repeat: None,
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "A2".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
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
                            repeat: None,
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B1".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B2".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B3".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
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
                            repeat: None,
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "A2".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
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
}