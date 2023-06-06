pub mod builders;
pub mod graph;
pub mod state;

pub use self::builders::*;
pub use self::graph::*;
pub use self::state::*;

use crate::definitions::*;
use std::collections::HashSet;
use thiserror::Error;

pub const START_STEP_ID: &str = "_START_";
pub const END_STEP_ID: &str = "_END_";

pub type StepID = String;

impl Quest {
    /// Check if the quest has a step defined by its id
    pub fn contanins_step(&self, step_id: &StepID) -> bool {
        let Some(definition) = &self.definition else {
            return false;
        };
        definition.steps.iter().any(|step| step.id == *step_id)
    }

    /// Get step defined in the quest by its id
    pub fn get_step(&self, step_id: &StepID) -> Option<&Step> {
        let Some(definition) = &self.definition else {
            return None;
        };
        definition.steps.iter().find(|step| step.id == *step_id)
    }

    /// Get all steps in `connections` that don't have a connection defined in which they are the `from` pointing to other node.
    ///
    /// We use this in order to know which steps point to the end node
    ///
    pub(crate) fn get_steps_without_to(&self) -> HashSet<StepID> {
        let mut steps = HashSet::new();
        let Some(definition) = &self.definition else {
            return steps;
        };
        for connection in &definition.connections {
            if definition
                .connections
                .iter()
                .all(|conn| conn.step_from != connection.step_to)
            {
                steps.insert(connection.step_to.clone());
            }
        }

        steps
    }

    /// Get all steps in `connections` that don't have a connection defined in which they are the `to`.
    ///
    /// We use this in order to know which steps are possible starting points
    ///
    pub(crate) fn get_steps_without_from(&self) -> HashSet<StepID> {
        let mut steps = HashSet::new();
        let Some(definition) = &self.definition else {
            return steps;
        };
        for connection in &definition.connections {
            if definition
                .connections
                .iter()
                .all(|conn| conn.step_to != connection.step_from)
            {
                steps.insert(connection.step_from.clone());
            }
        }

        steps
    }

    /// Validates a Quest struct to check if it meets all the requirements to be a valid quests
    ///
    pub fn is_valid(&self) -> Result<(), QuestValidationError> {
        let Some(definition) = &self.definition else {
            return Err(QuestValidationError::InvalidDefinition);
        };
        if definition.connections.is_empty() || definition.steps.is_empty() {
            return Err(QuestValidationError::InvalidDefinition);
        }
        let starting_nodes = self.get_steps_without_from();
        // Has at least one node for starting.
        // Note: This should be impossible
        if starting_nodes.is_empty() {
            return Err(QuestValidationError::NoStartingNode);
        }
        // All starting nodes should be defined as Step
        for step_id in starting_nodes {
            if !self.contanins_step(&step_id) {
                return Err(QuestValidationError::MissingStepForStartingNode(step_id));
            }
        }
        let end_nodes = self.get_steps_without_to();
        // Has at least one node pointing to end
        // Note: This should be impossible
        if end_nodes.is_empty() {
            return Err(QuestValidationError::NoEndNode);
        }
        // All end nodes should be defined as Step
        for step_id in end_nodes {
            if !self.contanins_step(&step_id) {
                return Err(QuestValidationError::MissingStepForEndNode(step_id));
            }
        }

        // Used to check all steps/tasks have a unique ID
        let mut unique_task_ids: HashSet<String> = HashSet::new();
        let mut unique_step_ids: HashSet<String> = HashSet::new();

        for step in &definition.steps {
            // All steps should not contain Tasks::None used for START and END nodes
            if step.tasks.is_empty() {
                return Err(QuestValidationError::MissingTasksForStep(step.id.clone()));
            }

            if !unique_step_ids.insert(step.id.to_string()) {
                // Step with same id has been seen
                return Err(QuestValidationError::NotUniqueIDForStep(step.id.clone()));
            }

            // All steps tasks (if there) have unique ID
            for task in &step.tasks {
                if !unique_task_ids.insert(task.id.to_string()) {
                    // Task with same id has been seen
                    return Err(QuestValidationError::NotUniqueIDForStepTask(
                        step.id.clone(),
                    ));
                }
            }

            // All steps have at least one defined connection
            if !definition
                .connections
                .iter()
                .any(|connection| connection.step_from == step.id || connection.step_to == step.id)
            {
                return Err(QuestValidationError::NoConnectionDefinedForStep(
                    step.id.clone(),
                ));
            }
        }

        // All connection halfs have a defined step
        for Connection {
            ref step_from,
            ref step_to,
        } in &definition.connections
        {
            if !self.contanins_step(step_from) {
                return Err(QuestValidationError::NoStepDefinedForConnectionHalf(
                    step_from.clone(),
                ));
            }

            if !self.contanins_step(step_to) {
                return Err(QuestValidationError::NoStepDefinedForConnectionHalf(
                    step_to.clone(),
                ));
            }
        }

        Ok(())
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum QuestValidationError {
    /// Definition is not valid because it has defined no connections or steps
    #[error("Missing the definition for the quest")]
    InvalidDefinition,
    /// No node to start the quest
    ///
    /// Note: This should be impossible but we do the check
    #[error("Missing a starting node for the quest")]
    NoStartingNode,
    /// No node pointing to end
    ///
    /// Note: This should be impossible but we do the check
    #[error("Missing a end node for the quest")]
    NoEndNode,
    /// One starting node doesn't have a defined step
    #[error(
        "Missing a definited step for the starting node defined in connections - Step ID: {0}"
    )]
    MissingStepForStartingNode(StepID),
    /// One end node doesn't have a defined step
    #[error("Missing a definited step for the end node defined in connections - Step ID: {0}")]
    MissingStepForEndNode(StepID),
    /// A Step doesn't have a defined connection
    #[error("Step has no connection - Step ID: {0}")]
    NoConnectionDefinedForStep(StepID),
    /// A Half of a connection tuple doesn't have a step defined
    #[error("Connection half has no defined step - Step ID: {0}")]
    NoStepDefinedForConnectionHalf(StepID),
    /// Not unique ID for the Step
    #[error("Step ID is not unique - Step ID: {0}")]
    NotUniqueIDForStep(StepID),
    /// Not unique ID for the Subtask
    #[error("Step's Task ID is not unique - Step ID: {0}")]
    NotUniqueIDForStepTask(StepID),
    /// Step should not has Tasks::None
    #[error("Step {0} doesn't have tasks defined")]
    MissingTasksForStep(StepID),
}

impl Connection {
    pub fn new(step_from: &str, step_to: &str) -> Self {
        Self {
            step_from: step_from.to_string(),
            step_to: step_to.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_starting_steps_properly() {
        let quest = Quest {
            id: "".to_string(),
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: Some(QuestDefinition {
                connections: vec![
                    Connection::new("A1", "B"),
                    Connection::new("B", "C"),
                    Connection::new("A2", "D"),
                    Connection::new("A3", "E"),
                ],
                steps: vec![], // not needed for test
            }),
        };

        let starting_steps = quest.get_steps_without_from();
        assert_eq!(starting_steps.len(), 3);
        assert!(starting_steps.contains(&"A1".to_string()));
        assert!(starting_steps.contains(&"A2".to_string()));
        assert!(starting_steps.contains(&"A3".to_string()));
    }

    #[test]
    fn get_steps_pointing_to_end_properly() {
        let quest = Quest {
            id: "".to_string(),
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: Some(QuestDefinition {
                connections: vec![
                    Connection::new("A1", "B"),
                    Connection::new("B", "C"),
                    Connection::new("A2", "D"),
                    Connection::new("A3", "E"),
                ],
                steps: vec![], // not needed for test
            }),
        };

        let steps_pointing_to_end = quest.get_steps_without_to();
        assert_eq!(steps_pointing_to_end.len(), 3);
        assert!(steps_pointing_to_end.contains(&"C".to_string()));
        assert!(steps_pointing_to_end.contains(&"D".to_string()));
        assert!(steps_pointing_to_end.contains(&"E".to_string()));
    }

    #[test]
    fn quest_should_be_valid() {
        let quest = Quest {
            id: "".to_string(),
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: Some(QuestDefinition {
                connections: vec![Connection::new("A1", "B"), Connection::new("B", "C")],
                steps: vec![
                    Step {
                        id: "A1".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            action_items: vec![Action::location(Coordinates::new(10, 10))],
                            id: "A1_1".to_string(),
                            description: "".to_string(),
                        }],
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            action_items: vec![Action::location(Coordinates::new(10, 15))],
                            id: "B_1".to_string(),
                            description: "".to_string(),
                        }],
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            action_items: vec![Action::location(Coordinates::new(10, 20))],
                            id: "C_1".to_string(),
                            description: "".to_string(),
                        }],
                    },
                ],
            }),
        };

        assert!(quest.is_valid().is_ok())
    }

    #[test]
    fn quest_should_not_be_valid() {
        // Should not be valid because of missing connections and steps
        let quest = Quest {
            id: "".to_string(),
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: Some(QuestDefinition {
                connections: vec![],
                steps: vec![], // not needed for test
            }),
        };
        let assert = matches!(
            quest.is_valid().unwrap_err(),
            QuestValidationError::InvalidDefinition
        );
        assert!(assert);

        // Should not be valid because of missing step for starting ndoe
        let quest = Quest {
            id: "".to_string(),
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: Some(QuestDefinition {
                connections: vec![Connection::new("A1", "B")],
                steps: vec![Step {
                    id: "B".to_string(),
                    description: "".to_string(),
                    tasks: vec![Task {
                        id: "B_1".to_string(),
                        description: "".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 15))],
                    }],
                }],
            }),
        };
        let err = QuestValidationError::MissingStepForStartingNode("A1".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        // Should not be valid because of missing step for end ndoe
        let quest = Quest {
            id: "".to_string(),
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: Some(QuestDefinition {
                connections: vec![Connection::new("A1", "B")],
                steps: vec![Step {
                    id: "A1".to_string(),
                    description: "".to_string(),
                    tasks: vec![Task {
                        id: "A1_1".to_string(),
                        description: "".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 15))],
                    }],
                }],
            }),
        };
        let err = QuestValidationError::MissingStepForEndNode("B".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        // Should not be valid because of missing connection for a defined step
        let quest = Quest {
            id: "".to_string(),

            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: Some(QuestDefinition {
                connections: vec![Connection::new("B", "C")],
                steps: vec![
                    Step {
                        id: "A1".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "A1_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![Action::location(Coordinates::new(10, 15))],
                        }],
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "B_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![Action::location(Coordinates::new(20, 15))],
                        }],
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "C_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![Action::location(Coordinates::new(10, 25))],
                        }],
                    },
                ],
            }),
        };
        let err = QuestValidationError::NoConnectionDefinedForStep("A1".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        // Should not be valid because of missing step for a defined connection
        let quest = Quest {
            id: "".to_string(),
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: Some(QuestDefinition {
                connections: vec![Connection::new("A1", "B"), Connection::new("B", "C")],
                steps: vec![
                    Step {
                        id: "A1".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "A1_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![Action::location(Coordinates::new(20, 15))],
                        }],
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "C_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![Action::location(Coordinates::new(30, 15))],
                        }],
                    },
                ],
            }),
        };
        let err = QuestValidationError::NoStepDefinedForConnectionHalf("B".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        // Should not be valid because of repeated ID for step
        let quest = Quest {
            id: "".to_string(),
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: Some(QuestDefinition {
                connections: vec![Connection::new("A", "B"), Connection::new("B", "C")],
                steps: vec![
                    Step {
                        id: "A".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "A_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![Action::location(Coordinates::new(10, 15))],
                        }],
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "B_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![Action::location(Coordinates::new(20, 15))],
                        }],
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "C_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![Action::location(Coordinates::new(10, 2))],
                        }],
                    },
                    Step {
                        id: "A".to_string(),
                        description: "Another A".to_string(),
                        tasks: vec![Task {
                            id: "A_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![Action::location(Coordinates::new(1, 15))],
                        }],
                    },
                ],
            }),
        };
        let err = QuestValidationError::NotUniqueIDForStep("A".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        // Should not be valid because of repeated ID on subtasks
        let quest = Quest {
            id: "".to_string(),
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
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
                                action_items: vec![Action::location(Coordinates::new(10, 20))],
                            },
                            Task {
                                id: "A_1".to_string(),
                                description: "".to_string(),
                                action_items: vec![Action::jump(Coordinates::new(30, 20))],
                            },
                        ],
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            id: "B_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![Action::location(Coordinates::new(20, 15))],
                        }],
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            action_items: vec![Action::location(Coordinates::new(10, 2))],
                            id: "C_1".to_string(),
                            description: "".to_string(),
                        }],
                    },
                ],
            }),
        };
        let err = QuestValidationError::NotUniqueIDForStepTask("A".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        // Should not be valid because of Tasks::None
        let quest = Quest {
            id: "".to_string(),
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
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
                                action_items: vec![Action::location(Coordinates::new(10, 20))],
                            },
                            Task {
                                id: "A_2".to_string(),
                                description: "".to_string(),
                                action_items: vec![Action::location(Coordinates::new(30, 20))],
                            },
                        ],
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: vec![],
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: vec![Task {
                            action_items: vec![Action::location(Coordinates::new(10, 2))],
                            id: "C_1".to_string(),
                            description: "".to_string(),
                        }],
                    },
                ],
            }),
        };
        let err = QuestValidationError::MissingTasksForStep("B".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);
    }
}
