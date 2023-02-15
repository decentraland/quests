use std::collections::HashSet;
use thiserror::Error;

use serde::{Deserialize, Serialize};

pub const START_STEP_ID: &str = "_START_";
pub const END_STEP_ID: &str = "_END_";

pub type StepID = String;

#[derive(Serialize, Deserialize, Debug)]
pub struct Quest {
    pub name: String,
    pub description: String,
    pub definition: QuestDefinition,
}

impl Quest {
    /// Check if the quest has a step defined by its id
    pub fn contanins_step(&self, step_id: &StepID) -> bool {
        self.definition.steps.iter().any(|step| step.id == *step_id)
    }

    /// Get step defined in the quest by its id
    pub fn get_step(&self, step_id: &StepID) -> Option<&Step> {
        self.definition
            .steps
            .iter()
            .find(|step| step.id == *step_id)
    }

    /// Get all steps in `connections` that don't have a connection defined in which they are the `from` pointing to other node.
    ///
    /// We use this in order to know which steps point to the end node
    ///
    pub(crate) fn get_steps_without_to(&self) -> HashSet<StepID> {
        let mut steps = HashSet::new();
        for connection in &self.definition.connections {
            if self
                .definition
                .connections
                .iter()
                .all(|conn| conn.0 != connection.1)
            {
                steps.insert(connection.1.clone());
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
        for connection in &self.definition.connections {
            if self
                .definition
                .connections
                .iter()
                .all(|conn| conn.1 != connection.0)
            {
                steps.insert(connection.0.clone());
            }
        }

        steps
    }

    /// Validates a Quest struct to check if it meets all the requirements to be a valid quests
    ///
    pub fn is_valid(&self) -> Result<(), QuestValidationError> {
        if self.definition.connections.is_empty() || self.definition.steps.is_empty() {
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

        for step in &self.definition.steps {
            // All steps should not contain Tasks::None used for START and END nodes
            if matches!(step.tasks, Tasks::None) {
                return Err(QuestValidationError::MissingTasksForStep(step.id.clone()));
            }

            // All steps has an unique ID
            if self
                .definition
                .steps
                .iter()
                .filter(|other_step| step.id == other_step.id)
                .count()
                > 1
            {
                return Err(QuestValidationError::NotUniqueIDForStep(step.id.clone()));
            }

            // All steps subtasks (if there) have unique ID
            // TODO: Find a way to check uniqueness between all steps' subtasks
            if let Tasks::Multiple(subtasks) = &step.tasks {
                for subtask in subtasks {
                    if subtasks.iter().filter(|s| s.id == subtask.id).count() > 1 {
                        return Err(QuestValidationError::NotUniqueIDForStepSubtask(
                            step.id.clone(),
                        ));
                    }
                }
            }

            // All steps have at least one defined connection
            if !self
                .definition
                .connections
                .iter()
                .any(|connection| connection.0 == step.id || connection.1 == step.id)
            {
                return Err(QuestValidationError::NoConnectionDefinedForStep(
                    step.id.clone(),
                ));
            }
        }

        // All connection halfs have a defined step
        for (from_id, to_id) in &self.definition.connections {
            if !self.contanins_step(from_id) {
                return Err(QuestValidationError::NoStepDefinedForConnectionHalf(
                    from_id.clone(),
                ));
            }

            if !self.contanins_step(to_id) {
                return Err(QuestValidationError::NoStepDefinedForConnectionHalf(
                    to_id.clone(),
                ));
            }
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QuestDefinition {
    pub steps: Vec<Step>,
    /// Connections between steps
    ///
    /// First position in the tuple is for `from` and second one `to`
    pub connections: Vec<(StepID, StepID)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Step {
    pub id: StepID,
    pub description: String,
    pub tasks: Tasks,
    /// Allow hooks on every completed step
    pub on_complete_hook: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Tasks {
    /// Step with only one stask
    Single {
        /// Required actions to complete the task
        action_items: Vec<Action>, // Loop = Multiple Actions
    },
    /// Step with multiple tasks to do in order to be completed
    Multiple(Vec<SubTask>),
    /// We only use this type for START and END nodes because we consider them as "Step"
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SubTask {
    pub id: String,
    pub description: String,
    /// Required actions to complete the task
    pub action_items: Vec<Action>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub address: String,
    pub timestamp: usize,
    pub action: Action,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Coordinates(pub usize, pub usize);

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
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
    #[error("Step's Subtask ID is not unique - Step ID: {0}")]
    NotUniqueIDForStepSubtask(StepID),
    /// Step should not has Tasks::None
    #[error("Step {0} doesn't have tasks defined")]
    MissingTasksForStep(StepID),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_starting_steps_properly() {
        let quest = Quest {
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: QuestDefinition {
                connections: vec![
                    ("A1".to_string(), "B".to_string()),
                    ("B".to_string(), "C".to_string()),
                    ("A2".to_string(), "D".to_string()),
                    ("A3".to_string(), "E".to_string()),
                ],
                steps: vec![], // not needed for test
            },
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
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: QuestDefinition {
                connections: vec![
                    ("A1".to_string(), "B".to_string()),
                    ("B".to_string(), "C".to_string()),
                    ("A2".to_string(), "D".to_string()),
                    ("A3".to_string(), "E".to_string()),
                ],
                steps: vec![], // not needed for test
            },
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
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: QuestDefinition {
                connections: vec![
                    ("A1".to_string(), "B".to_string()),
                    ("B".to_string(), "C".to_string()),
                ],
                steps: vec![
                    Step {
                        id: "A1".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::Location {
                                coordinates: Coordinates(10, 10),
                            }],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::Location {
                                coordinates: Coordinates(10, 15),
                            }],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::Location {
                                coordinates: Coordinates(10, 20),
                            }],
                        },
                        on_complete_hook: None,
                    },
                ],
            },
        };

        assert!(quest.is_valid().is_ok())
    }

    #[test]
    fn quest_should_not_be_valid() {
        // Should not be valid because of missing connections and steps
        let quest = Quest {
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: QuestDefinition {
                connections: vec![],
                steps: vec![], // not needed for test
            },
        };
        let assert = matches!(
            quest.is_valid().unwrap_err(),
            QuestValidationError::InvalidDefinition
        );
        assert!(assert);

        // Should not be valid because of missing step for starting ndoe
        let quest = Quest {
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: QuestDefinition {
                connections: vec![("A1".to_string(), "B".to_string())],
                steps: vec![Step {
                    id: "B".to_string(),
                    description: "".to_string(),
                    tasks: Tasks::Single {
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(10, 15),
                        }],
                    },
                    on_complete_hook: None,
                }],
            },
        };
        let err = QuestValidationError::MissingStepForStartingNode("A1".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        // Should not be valid because of missing step for end ndoe
        let quest = Quest {
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: QuestDefinition {
                connections: vec![("A1".to_string(), "B".to_string())],
                steps: vec![Step {
                    id: "A1".to_string(),
                    description: "".to_string(),
                    tasks: Tasks::Single {
                        action_items: vec![Action::Location {
                            coordinates: Coordinates(10, 15),
                        }],
                    },
                    on_complete_hook: None,
                }],
            },
        };
        let err = QuestValidationError::MissingStepForEndNode("B".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        // Should not be valid because of missing connection for a defined step
        let quest = Quest {
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: QuestDefinition {
                connections: vec![("B".to_string(), "C".to_string())],
                steps: vec![
                    Step {
                        id: "A1".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::Location {
                                coordinates: Coordinates(10, 15),
                            }],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::Location {
                                coordinates: Coordinates(20, 15),
                            }],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::Location {
                                coordinates: Coordinates(10, 25),
                            }],
                        },
                        on_complete_hook: None,
                    },
                ],
            },
        };
        let err = QuestValidationError::NoConnectionDefinedForStep("A1".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        // Should not be valid because of missing step for a defined connection
        let quest = Quest {
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: QuestDefinition {
                connections: vec![
                    ("A1".to_string(), "B".to_string()),
                    ("B".to_string(), "C".to_string()),
                ],
                steps: vec![
                    Step {
                        id: "A1".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::Location {
                                coordinates: Coordinates(20, 15),
                            }],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::Location {
                                coordinates: Coordinates(30, 15),
                            }],
                        },
                        on_complete_hook: None,
                    },
                ],
            },
        };
        let err = QuestValidationError::NoStepDefinedForConnectionHalf("B".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        // Should not be valid because of repeated ID for step
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
                        tasks: Tasks::Single {
                            action_items: vec![Action::Location {
                                coordinates: Coordinates(10, 15),
                            }],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::Location {
                                coordinates: Coordinates(20, 15),
                            }],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::Location {
                                coordinates: Coordinates(10, 2),
                            }],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "A".to_string(),
                        description: "Another A".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::Location {
                                coordinates: Coordinates(1, 15),
                            }],
                        },
                        on_complete_hook: None,
                    },
                ],
            },
        };
        let err = QuestValidationError::NotUniqueIDForStep("A".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        // Should not be valid because of repeated ID on subtasks
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
                                action_items: vec![Action::Location {
                                    coordinates: Coordinates(10, 20),
                                }],
                            },
                            SubTask {
                                id: "A_1".to_string(),
                                description: "".to_string(),
                                action_items: vec![Action::Jump {
                                    coordinates: Coordinates(30, 20),
                                }],
                            },
                        ]),
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::Location {
                                coordinates: Coordinates(20, 15),
                            }],
                        },
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::Location {
                                coordinates: Coordinates(10, 2),
                            }],
                        },
                        on_complete_hook: None,
                    },
                ],
            },
        };
        let err = QuestValidationError::NotUniqueIDForStepSubtask("A".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        // Should not be valid because of tasks:None
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
                                action_items: vec![Action::Location {
                                    coordinates: Coordinates(10, 20),
                                }],
                            },
                            SubTask {
                                id: "A_2".to_string(),
                                description: "".to_string(),
                                action_items: vec![Action::Jump {
                                    coordinates: Coordinates(30, 20),
                                }],
                            },
                        ]),
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::None,
                        on_complete_hook: None,
                    },
                    Step {
                        id: "C".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::Single {
                            action_items: vec![Action::Location {
                                coordinates: Coordinates(10, 2),
                            }],
                        },
                        on_complete_hook: None,
                    },
                ],
            },
        };
        let err = QuestValidationError::MissingTasksForStep("B".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);
    }
}
