use crate::definitions::*;
use std::{collections::HashMap, hash::Hash};

use super::*;
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
    pub tasks_by_step: HashMap<StepID, Vec<Task>>,
}

impl QuestGraph {
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

impl From<&Quest> for QuestGraph {
    fn from(value: &Quest) -> Self {
        Self {
            graph: build_graph_from_quest_definition(value),
            tasks_by_step: build_tasks_by_step_from_quest_definition(value),
        }
    }
}

fn build_graph_from_quest_definition(quest: &Quest) -> Dag<String, u32> {
    let mut dag = daggy::Dag::<String, u32, u32>::new();
    let starting_step = Step {
        id: START_STEP_ID.to_string(),
        description: "COMMON START NODE".to_string(),
        tasks: vec![],
    };
    let ending_step = Step {
        id: END_STEP_ID.to_string(),
        description: "COMMON END NODE".to_string(),
        tasks: vec![],
    };

    let start_node = dag.add_node(starting_step.id);
    let end_node = dag.add_node(ending_step.id);
    let mut nodes: HashMap<String, NodeIndex> = HashMap::new();

    let Some(definition) = &quest.definition else {
        return dag;
    };
    for Connection {
        ref step_from,
        ref step_to,
    } in &definition.connections
    {
        // Validate if steps are in defined in the quest
        if quest.contains_step(step_from) && quest.contains_step(step_to) {
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

fn build_tasks_by_step_from_quest_definition(quest: &Quest) -> HashMap<StepID, Vec<Task>> {
    let mut content_map = HashMap::new();
    let Some(definition) = &quest.definition else {
        return content_map;
    };
    for step in &definition.steps {
        let content = step.tasks.clone();
        content_map.insert(step.id.clone(), content);
    }
    content_map
}

fn keys_match<T: Eq + Hash, U, V>(map1: &HashMap<T, U>, map2: &HashMap<T, V>) -> bool {
    map1.len() == map2.len() && map1.keys().all(|k| map2.contains_key(k))
}

/// Case insensitive check on action type and parameters
pub fn matches_action(action: &Action, other_action: &Action) -> bool {
    if !action.r#type.eq_ignore_ascii_case(&other_action.r#type) {
        return false;
    };

    if !keys_match(&action.parameters, &other_action.parameters) {
        return false;
    };

    for (key, value) in &action.parameters {
        let other_value = if let Some(other_value) = other_action.parameters.get(key) {
            other_value
        } else {
            return false;
        };
        if !value.eq_ignore_ascii_case(other_value) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::builders::{Coordinates, EMOTE};
    use super::*;

    #[test]
    fn build_quest_graph_properly() {
        let quest = Quest {
            id: "1e9a8bbf-2223-4f51-b7e5-660d35cedef4".to_string(),
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
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
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "A_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "B_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "C_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "D_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![],
                        }],
                    },
                ],
            }),
            ..Default::default()
        };

        let graph: QuestGraph = (&quest).into();

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
            id: "1e9a8bbf-2223-4f51-b7e5-660d35cedef4".to_string(),
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
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "A2".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "A2_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "B_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "C_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "D_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![],
                        }],
                    },
                ],
            }),
            ..Default::default()
        };

        let graph = QuestGraph::from(&quest);
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
            id: "1e9a8bbf-2223-4f51-b7e5-660d35cedef4".to_string(),
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            creator_address: "0xB".to_string(),
            definition: Some(QuestDefinition {
                connections: vec![
                    Connection::new("A", "B1"),
                    Connection::new("A", "B2"),
                    Connection::new("A", "B3"),
                    Connection::new("B1", "C"),
                    Connection::new("C", "D"),
                ],
                steps: vec![
                    Step {
                        id: "A".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "A_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "B1".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "B_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "B2".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "B_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "B3".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "B_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "C_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "D_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![],
                        }],
                    },
                ],
            }),
            ..Default::default()
        };

        let graph = QuestGraph::from(&quest);
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
            id: "1e9a8bbf-2223-4f51-b7e5-660d35cedef4".to_string(),
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
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "A2".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "A2_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "B_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "C_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "D_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![],
                        }],
                    },
                ],
            }),
            ..Default::default()
        };

        let graph = QuestGraph::from(&quest);

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
            id: "1e9a8bbf-2223-4f51-b7e5-660d35cedef4".to_string(),
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
                        description: "A1 desc".to_string(),
                        tasks: vec![Task {
                            id: "A1_1".to_string(),
                            description: "A1_1".to_string(),
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "A2".to_string(),
                        description: "A2 desc".to_string(),
                        tasks: vec![Task {
                            id: "A2_1".to_string(),
                            description: "A_2 desc".to_string(),
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "B".to_string(),
                        description: "B Desc".to_string(),
                        tasks: vec![Task {
                            id: "B_1".to_string(),
                            description: "B_1 desc".to_string(),
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "C".to_string(),
                        description: "C desc".to_string(),
                        tasks: vec![Task {
                            id: "C_1".to_string(),
                            description: "C_1 desc".to_string(),
                            action_items: vec![],
                        }],
                    },
                    Step {
                        id: "D".to_string(),
                        description: "D desc".to_string(),
                        tasks: vec![Task {
                            id: "D_1".to_string(),
                            description: "D_1 desc".to_string(),
                            action_items: vec![],
                        }],
                    },
                ],
            }),
            ..Default::default()
        };

        assert!(quest.is_valid().is_ok());
        let graph = QuestGraph::from(&quest);
        let steps_required_for_end = graph.required_for_end().unwrap();
        assert!(steps_required_for_end.contains(&"D".to_string()));
        assert!(steps_required_for_end.contains(&"C".to_string()));
    }

    #[test]
    fn matches_action_works() {
        let result = matches_action(
            &Action::location(Coordinates::new(10, 10)),
            &Action::location(Coordinates::new(10, 10)),
        );
        assert!(result);

        let result = matches_action(
            &Action::location(Coordinates::new(10, 10)),
            &Action::location(Coordinates::new(10, 20)),
        );
        assert!(!result);

        let result = matches_action(
            &Action::location(Coordinates::new(10, 10)),
            &Action::jump(Coordinates::new(10, 10)),
        );
        assert!(!result);

        let result = matches_action(
            &Action::npc_interaction("NPC_ID"),
            &Action::npc_interaction("NPC_ID_2"),
        );
        assert!(!result);

        let result = matches_action(
            &Action::npc_interaction("NPC_ID"),
            &Action::npc_interaction("NPC_ID"),
        );
        assert!(result);

        let result = matches_action(
            &Action::emote(Coordinates::new(1, 2), "ID"),
            &Action::emote(Coordinates::new(1, 2), "ID"),
        );
        assert!(result);
        let mut other_action = Action::emote(Coordinates::new(1, 2), "ID");
        other_action.r#type = EMOTE.to_string().to_lowercase();
        let result = matches_action(&Action::emote(Coordinates::new(1, 2), "ID"), &other_action);
        assert!(result);
    }
}
