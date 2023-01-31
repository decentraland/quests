pub mod quests {
    use serde::{Deserialize, Serialize};

    pub const START_STEP_ID: &str = "_START_";
    pub const END_STEP_ID: &str = "_END_";

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Quest {
        pub name: String,
        pub description: String,
        pub steps: Vec<Step>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Step {
        pub id: String,
        pub description: String,
        pub tasks: Tasks,
        /// Starting point containing the next step id. If it's none, it's just another step.
        pub pivot_point: Option<PivotPoint>,
        /// Allow hooks on every completed step
        pub on_complete_hook: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct PivotPoint {
        pub next_steps_id: PivotType,
        pub is_starting_point: bool,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub enum PivotType {
        OnePath(Vec<String>),
        MultiPath(Vec<String>),
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub enum Tasks {
        Single {
            /// Required actions to complete the task
            action_items: Vec<Action>,
            /// Looping task
            repeat: Option<u32>,
        },
        Multiple(Vec<SubTask>),
        None,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct SubTask {
        pub title: String,
        pub description: String,
        /// Required actions to complete the task
        pub action_items: Vec<Action>,
        /// Looping task
        pub repeat: Option<u32>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Event {
        pub address: String,
        pub timestamp: usize,
        pub action: Action,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Coordinates(usize, usize);

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum Action {
        Location {
            coordinates: Coordinates,
        },
        Jump {
            coordinates: Coordinates,
        },
        Emote {
            coordinates: Coordinates,
            emote_id: String,
        },
        NPCInteraction {
            npc_id: String,
        },
        Custom {
            id: String,
        },
    }
}

pub mod quest_graph {

    use super::quests::*;
    use daggy::{
        self,
        petgraph::dot::{Config, Dot},
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
            pivot_point: None,
        };
        let ending_step = Step {
            id: END_STEP_ID.to_string(),
            description: "COMMON END NODE".to_string(),
            tasks: Tasks::None,
            on_complete_hook: None,
            pivot_point: None,
        };

        let start_node = dag.add_node(starting_step.id);
        let end_node = dag.add_node(ending_step.id);

        let starting_points = quest.steps.iter().filter(|step| {
            if let Some(pivot_point) = &step.pivot_point {
                pivot_point.is_starting_point
            } else {
                false
            }
        });

        // Attach Quest's Starting points
        for starting_step_point in starting_points {
            let (_, current_start_node) = dag.add_child(
                start_node,
                start_node.index() as u32,
                starting_step_point.id.clone(),
            );
            if let Some(PivotPoint {
                next_steps_id,
                is_starting_point: _,
            }) = &starting_step_point.pivot_point
            {
                build_pivot_points(current_start_node, end_node, quest, &mut dag, next_steps_id)
            }
        }

        dag
    }

    fn build_pivot_points(
        current_start_node: NodeIndex,
        end_node: NodeIndex,
        quest: &Quest,
        dag: &mut Dag<String, u32>,
        next_step_ids: &PivotType,
    ) {
        match next_step_ids {
            PivotType::OnePath(next_ids) => {
                let mut last_node = current_start_node;
                for id in next_ids {
                    let step = quest.steps.iter().find(|step| *step.id == *id);
                    if let Some(step) = step {
                        let (_, current) =
                            dag.add_child(last_node, last_node.index() as u32, step.id.clone());
                        last_node = current;
                    }
                }
                dag.add_edge(last_node, end_node, 0).unwrap();
            }
            PivotType::MultiPath(next_ids) => {
                for id in next_ids {
                    let step = quest.steps.iter().find(|step| *step.id == *id);
                    if let Some(step) = step {
                        let (_, current) = dag.add_child(
                            current_start_node,
                            current_start_node.index() as u32,
                            step.id.clone(),
                        );
                        if let Some(PivotPoint {
                            next_steps_id,
                            is_starting_point: _,
                        }) = &step.pivot_point
                        {
                            build_pivot_points(current, end_node, quest, dag, next_steps_id)
                        } else {
                            dag.add_edge(current, end_node, 0).unwrap();
                        }
                    }
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn build_quest_graph_properly() {
            let quest = Quest {
                name: "CUSTOM_QUEST".to_string(),
                description: "".to_string(),
                steps: vec![
                    Step {
                        id: "A".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: Some(PivotPoint {
                            next_steps_id: PivotType::OnePath(vec![
                                "B".to_string(),
                                "C".to_string(),
                                "D".to_string(),
                            ]),
                            is_starting_point: true,
                        }),
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: None,
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: None,
                    },
                ],
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
                steps: vec![
                    Step {
                        id: "A1".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: Some(PivotPoint {
                            next_steps_id: PivotType::OnePath(vec![
                                "B".to_string(),
                                "C".to_string(),
                            ]),
                            is_starting_point: true,
                        }),
                    },
                    Step {
                        id: "A2".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: Some(PivotPoint {
                            next_steps_id: PivotType::OnePath(vec!["D".to_string()]),
                            is_starting_point: true,
                        }),
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: None,
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: None,
                    },
                ],
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
                steps: vec![
                    Step {
                        id: "A".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: Some(PivotPoint {
                            next_steps_id: PivotType::MultiPath(vec![
                                "B1".to_string(),
                                "B2".to_string(),
                                "B3".to_string(),
                            ]),
                            is_starting_point: true,
                        }),
                    },
                    Step {
                        id: "B1".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: Some(PivotPoint {
                            next_steps_id: PivotType::OnePath(vec![
                                "C".to_string(),
                                "D".to_string(),
                            ]),
                            is_starting_point: false,
                        }),
                    },
                    Step {
                        id: "B2".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: None,
                    },
                    Step {
                        id: "B3".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: None,
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: None,
                    },
                ],
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
                steps: vec![
                    Step {
                        id: "A1".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: Some(PivotPoint {
                            next_steps_id: PivotType::OnePath(vec![
                                "B".to_string(),
                                "C".to_string(),
                            ]),
                            is_starting_point: true,
                        }),
                    },
                    Step {
                        id: "A2".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: Some(PivotPoint {
                            next_steps_id: PivotType::OnePath(vec!["D".to_string()]),
                            is_starting_point: true,
                        }),
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: None,
                    },
                    Step {
                        id: "D".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![],
                            repeat: None,
                        },
                        on_complete_hook: None,
                        pivot_point: None,
                    },
                ],
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
}
