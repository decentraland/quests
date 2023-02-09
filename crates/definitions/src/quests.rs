use std::collections::HashSet;

use serde::{Deserialize, Serialize};

pub const START_STEP_ID: &str = "_START_";
pub const END_STEP_ID: &str = "_END_";

type StepID = String;

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
    /// *TODO: add validations for Tasks, Actions, and `on_complete_hook`*
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

        // All steps have at least one defined connection
        for step in &self.definition.steps {
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
            if !self.contanins_step(from_id) || !self.contanins_step(to_id) {
                return Err(QuestValidationError::NoStepDefinedForConnectionHalf((
                    from_id.clone(),
                    to_id.clone(),
                )));
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Coordinates(usize, usize);

#[derive(Serialize, Deserialize, Debug)]
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

pub enum QuestValidationError {
    /// Definition is not valid because it has defined no connections or steps
    InvalidDefinition,
    /// No node to start the quest
    ///
    /// Note: This should be impossible but we do the check
    NoStartingNode,
    /// No node pointing to end
    ///
    /// Note: This should be impossible but we do the check
    NoEndNode,
    /// One starting node doesn't have a defined step
    MissingStepForStartingNode(StepID),
    /// One end node doesn't have a defined step
    MissingStepForEndNode(StepID),
    /// A Step doesn't have a defined connection
    NoConnectionDefinedForStep(StepID),
    /// A Half of a connection tuple doesn't have a step defined
    NoStepDefinedForConnectionHalf((StepID, StepID)),
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
                        tasks: Tasks::None,
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
                        tasks: Tasks::None,
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
                    tasks: Tasks::None,
                    on_complete_hook: None,
                }],
            },
        };
        let _err = QuestValidationError::MissingStepForStartingNode("A1".to_string());
        let assert = matches!(quest.is_valid().unwrap_err(), _err);
        assert!(assert);

        // Should not be valid because of missing step for end ndoe
        let quest = Quest {
            name: "CUSTOM_QUEST".to_string(),
            description: "".to_string(),
            definition: QuestDefinition {
                connections: vec![("A1".to_string(), "B".to_string())],
                steps: vec![Step {
                    id: "A1".to_string(),
                    description: "".to_string(),
                    tasks: Tasks::None,
                    on_complete_hook: None,
                }],
            },
        };
        let _err = QuestValidationError::MissingStepForEndNode("B".to_string());
        let assert = matches!(quest.is_valid().unwrap_err(), _err);
        assert!(assert);

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
                        tasks: Tasks::None,
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
                        tasks: Tasks::None,
                        on_complete_hook: None,
                    },
                ],
            },
        };
        let _err = QuestValidationError::NoConnectionDefinedForStep("A1".to_string());
        let assert = matches!(quest.is_valid().unwrap_err(), _err);
        assert!(assert);

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
                        tasks: Tasks::None,
                        on_complete_hook: None,
                    },
                    Step {
                        id: "B".to_string(),
                        description: "".to_string(),
                        tasks: Tasks::None,
                        on_complete_hook: None,
                    },
                ],
            },
        };
        let _err = QuestValidationError::NoStepDefinedForConnectionHalf((
            "B".to_string(),
            "C".to_string(),
        ));
        let assert = matches!(quest.is_valid().unwrap_err(), _err);
        assert!(assert);
    }
}
