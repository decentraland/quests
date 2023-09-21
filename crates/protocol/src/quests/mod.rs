pub mod builders;
pub mod graph;
pub mod state;

pub use self::builders::*;
pub use self::graph::*;
pub use self::state::*;

use crate::definitions::*;
use std::collections::HashMap;
use std::collections::HashSet;
use thiserror::Error;

pub const START_STEP_ID: &str = "_START_";
pub const END_STEP_ID: &str = "_END_";

pub type StepID = String;

impl Quest {
    /// Check if the quest has a step defined by its id
    pub fn contains_step(&self, step_id: &StepID) -> bool {
        let Some(definition) = &self.definition else {
            return false;
        };
        definition.contains_step(step_id)
    }

    /// Get step defined in the quest by its id
    pub fn get_step(&self, step_id: &StepID) -> Option<&Step> {
        let Some(definition) = &self.definition else {
            return None;
        };
        definition.get_step(step_id)
    }

    /// Get all steps in `connections` that don't have a connection defined in which they are the `from` pointing to other node.
    ///
    /// We use this in order to know which steps point to the end node
    ///
    pub(crate) fn get_steps_without_to(&self) -> HashSet<StepID> {
        let Some(definition) = &self.definition else {
            return HashSet::new();
        };
        definition.get_steps_without_to()
    }

    /// Get all steps in `connections` that don't have a connection defined in which they are the `to`.
    ///
    /// We use this in order to know which steps are possible starting points
    ///
    pub(crate) fn get_steps_without_from(&self) -> HashSet<StepID> {
        let Some(definition) = &self.definition else {
            return HashSet::new();
        };
        definition.get_steps_without_from()
    }

    /// Validates a Quest struct to check if it meets all the requirements to be a valid quests
    ///
    pub fn is_valid(&self) -> Result<(), QuestValidationError> {
        let Some(definition) = &self.definition else {
            return Err(QuestValidationError::InvalidDefinition);
        };
        definition.is_valid()
    }

    pub fn hide_actions(&mut self) {
        if let Some(definition) = self.definition.as_mut() {
            definition
                .steps
                .iter_mut()
                .for_each(|step| step.tasks.iter_mut().for_each(|task| task.hide_actions()));
        }
    }
}

impl QuestDefinition {
    pub fn is_valid(&self) -> Result<(), QuestValidationError> {
        let definition = self;
        if definition.steps.is_empty() {
            return Err(QuestValidationError::InvalidDefinition);
        }

        // All connection halfs have a defined step
        for Connection { step_from, step_to } in &definition.connections {
            if !self.contains_step(step_from) {
                return Err(QuestValidationError::MissingStepDefinition(
                    step_from.clone(),
                ));
            }

            if !self.contains_step(step_to) {
                return Err(QuestValidationError::MissingStepDefinition(step_to.clone()));
            }
        }

        // Has at least one node for starting.
        let starting_nodes = self.get_steps_without_from();
        if starting_nodes.is_empty() {
            return Err(QuestValidationError::NoStartingNode);
        }

        // Has at least one node pointing to end
        let end_nodes = self.get_steps_without_to();
        if end_nodes.is_empty() {
            return Err(QuestValidationError::NoEndNode);
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

            if step.description.is_empty() {
                return Err(QuestValidationError::MissingDescriptionForStep(
                    step.id.to_string(),
                ));
            }

            // All steps tasks (if there) have unique ID
            for task in &step.tasks {
                if !unique_task_ids.insert(task.id.to_string()) {
                    // Task with same id has been seen
                    return Err(QuestValidationError::NotUniqueIDForStepTask(
                        step.id.clone(),
                    ));
                }

                if task.description.is_empty() {
                    return Err(QuestValidationError::MissingDescriptionForTask(
                        task.id.to_string(),
                    ));
                }

                for action_item in &task.action_items {
                    match &*action_item.r#type {
                        "CUSTOM" => {
                            if action_item.parameters.keys().len() == 0 {
                                return Err(QuestValidationError::ActionItemParametersNotValid(
                                    "CUSTOM".to_string(),
                                ));
                            }
                        }
                        "LOCATION" => {
                            if action_item.parameters.get("x").is_none()
                                || action_item.parameters.get("y").is_none()
                            {
                                return Err(QuestValidationError::ActionItemParametersNotValid(
                                    "LOCATION".to_string(),
                                ));
                            }
                        }
                        "EMOTE" => {
                            if action_item.parameters.get("x").is_none()
                                || action_item.parameters.get("y").is_none()
                                || action_item.parameters.get("id").is_none()
                            {
                                return Err(QuestValidationError::ActionItemParametersNotValid(
                                    "EMOTE".to_string(),
                                ));
                            }
                        }
                        "JUMP" => {
                            if action_item.parameters.get("x").is_none()
                                || action_item.parameters.get("y").is_none()
                            {
                                return Err(QuestValidationError::ActionItemParametersNotValid(
                                    "JUMP".to_string(),
                                ));
                            }
                        }
                        _ => {
                            return Err(QuestValidationError::ActionItemTypeNotValid(
                                action_item.r#type.clone(),
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn contains_step(&self, step_id: &StepID) -> bool {
        self.steps.iter().any(|step| step.id == *step_id)
    }

    /// Get step defined in the quest by its id
    fn get_step(&self, step_id: &StepID) -> Option<&Step> {
        self.steps.iter().find(|step| step.id == *step_id)
    }

    /// Returns all the steps that don't appear as step from in a connection
    fn get_steps_without_to(&self) -> HashSet<StepID> {
        let mut steps = HashSet::new();
        let mut connections = HashMap::new();
        for connection in &self.connections {
            connections.insert(connection.step_from.clone(), connection.step_to.clone());
        }

        for step in &self.steps {
            if !connections.contains_key(&step.id) {
                steps.insert(step.id.clone());
            }
        }

        steps
    }

    /// Returns all the steps that don't appear as step to in a connection
    fn get_steps_without_from(&self) -> HashSet<StepID> {
        let mut steps = HashSet::new();
        let mut connections = HashMap::new();
        for connection in &self.connections {
            connections.insert(connection.step_to.clone(), connection.step_from.clone());
        }

        for step in &self.steps {
            if !connections.contains_key(&step.id) {
                steps.insert(step.id.clone());
            }
        }

        steps
    }
}

impl QuestState {
    pub fn hide_actions(&mut self) {
        for step in self.current_steps.values_mut() {
            step.hide_actions();
        }
    }
}

impl StepContent {
    pub fn hide_actions(&mut self) {
        for task in &mut self.to_dos {
            task.hide_actions();
        }
    }
}

impl Task {
    pub fn hide_actions(&mut self) {
        self.action_items.clear();
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
    /// Step must have a description
    #[error("Step must have a description: {0}")]
    MissingDescriptionForStep(String),
    /// Task must have a description
    #[error("Task must have a description: {0}")]
    MissingDescriptionForTask(String),
    /// A Half of a connection tuple doesn't have a step defined
    #[error("Connection half has no defined step - Step ID: {0}")]
    MissingStepDefinition(StepID),
    /// Not unique ID for the Step
    #[error("Step ID is not unique - Step ID: {0}")]
    NotUniqueIDForStep(StepID),
    /// Not unique ID for the Subtask
    #[error("Step's Task ID is not unique - Step ID: {0}")]
    NotUniqueIDForStepTask(StepID),
    /// Step should not has Tasks::None
    #[error("Step {0} doesn't have tasks defined")]
    MissingTasksForStep(StepID),
    /// Action Item type should be valid
    #[error("Action Item's type is not valid: {0}")]
    ActionItemTypeNotValid(String),
    /// Action Item parameters should be valid
    #[error("Action Item's parameters are not valid: {0}")]
    ActionItemParametersNotValid(String),
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

    struct QuestBuilder {
        steps: Vec<Step>,
        connections: Vec<Connection>,
    }

    impl QuestBuilder {
        fn new() -> Self {
            Self {
                steps: vec![],
                connections: vec![],
            }
        }

        fn with_steps(mut self, steps: Vec<Step>) -> Self {
            self.steps = steps;
            self
        }

        fn with_connections(mut self, connections: Vec<Connection>) -> Self {
            self.connections = connections;
            self
        }

        fn build(self) -> Quest {
            Quest {
                definition: Some(QuestDefinition {
                    connections: self.connections,
                    steps: self.steps,
                }),
                ..Default::default()
            }
        }
    }

    fn create_empty_step(name: &str) -> Step {
        Step {
            id: name.to_string(),
            description: "some desc".to_string(),
            tasks: vec![],
        }
    }

    fn create_simple_step(name: &str) -> Step {
        Step {
            id: name.to_string(),
            description: "some desc".to_string(),
            tasks: vec![Task {
                action_items: vec![Action::location(Coordinates::new(10, 10))],
                id: format!("{name}_1"),
                description: "task desc".to_string(),
            }],
        }
    }

    #[test]
    fn get_starting_steps_properly() {
        let quest = QuestBuilder::new()
            .with_connections(vec![
                Connection::new("A1", "B"),
                Connection::new("B", "C"),
                Connection::new("A2", "D"),
                Connection::new("A3", "E"),
            ])
            .with_steps(vec![
                create_empty_step("A1"),
                create_empty_step("A2"),
                create_empty_step("A3"),
                create_empty_step("B"),
                create_empty_step("C"),
                create_empty_step("D"),
                create_empty_step("E"),
            ])
            .build();

        let starting_steps = quest.get_steps_without_from();
        assert_eq!(starting_steps.len(), 3);
        assert!(starting_steps.contains(&"A1".to_string()));
        assert!(starting_steps.contains(&"A2".to_string()));
        assert!(starting_steps.contains(&"A3".to_string()));
    }

    #[test]
    fn get_steps_pointing_to_end_properly() {
        let quest = QuestBuilder::new()
            .with_connections(vec![
                Connection::new("A1", "B"),
                Connection::new("B", "C"),
                Connection::new("A2", "D"),
                Connection::new("A3", "E"),
            ])
            .with_steps(vec![
                create_empty_step("A1"),
                create_empty_step("A2"),
                create_empty_step("A3"),
                create_empty_step("B"),
                create_empty_step("C"),
                create_empty_step("D"),
                create_empty_step("E"),
            ])
            .build();

        let steps_pointing_to_end = quest.get_steps_without_to();
        assert_eq!(steps_pointing_to_end.len(), 3);
        assert!(steps_pointing_to_end.contains(&"C".to_string()));
        assert!(steps_pointing_to_end.contains(&"D".to_string()));
        assert!(steps_pointing_to_end.contains(&"E".to_string()));
    }

    #[test]
    fn quest_should_be_valid() {
        let quest = QuestBuilder::new()
            .with_connections(vec![Connection::new("A1", "B"), Connection::new("B", "C")])
            .with_steps(vec![
                create_simple_step("A1"),
                create_simple_step("B"),
                create_simple_step("C"),
            ])
            .build();

        assert!(quest.is_valid().is_ok());
    }

    #[test]
    fn multiple_steps_no_connections_quest_should_be_valid() {
        let quest = QuestBuilder::new()
            .with_steps(vec![
                create_simple_step("A"),
                create_simple_step("B"),
                create_simple_step("C"),
            ])
            .build();

        assert!(quest.is_valid().is_ok());
    }

    #[test]
    fn single_step_quest_should_be_valid() {
        let quest = QuestBuilder::new()
            .with_steps(vec![create_simple_step("A")])
            .build();

        assert!(quest.is_valid().is_ok());
    }

    #[test]
    fn quest_should_not_be_valid() {
        // Should not be valid because of missing steps
        let quest = QuestBuilder::new().build();
        let assert = matches!(
            quest.is_valid().unwrap_err(),
            QuestValidationError::InvalidDefinition
        );
        assert!(assert);

        // Should not be valid because of missing step for starting ndoe
        let quest = QuestBuilder::new()
            .with_connections(vec![Connection::new("A1", "B")])
            .with_steps(vec![create_simple_step("B")])
            .build();
        let err = QuestValidationError::MissingStepDefinition("A1".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        // Should not be valid because of missing step for end node
        let quest = QuestBuilder::new()
            .with_connections(vec![Connection::new("A1", "B")])
            .with_steps(vec![create_simple_step("A1")])
            .build();

        let err = QuestValidationError::MissingStepDefinition("B".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        // Should not be valid because of missing step for a defined connection
        let quest = QuestBuilder::new()
            .with_connections(vec![Connection::new("A1", "B"), Connection::new("B", "C")])
            .with_steps(vec![create_simple_step("A1"), create_simple_step("C")])
            .build();
        let err = QuestValidationError::MissingStepDefinition("B".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        // Should not be valid because of repeated ID for step
        let quest = QuestBuilder::new()
            .with_connections(vec![Connection::new("A", "B"), Connection::new("B", "C")])
            .with_steps(vec![
                create_simple_step("A"),
                create_simple_step("B"),
                create_simple_step("C"),
                create_simple_step("A"),
            ])
            .build();
        let err = QuestValidationError::NotUniqueIDForStep("A".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        // Should not be valid because of repeated ID on subtasks
        let quest = QuestBuilder::new()
            .with_connections(vec![Connection::new("A", "B"), Connection::new("B", "C")])
            .with_steps(vec![
                Step {
                    id: "A".to_string(),
                    description: "desc".to_string(),

                    tasks: vec![
                        Task {
                            id: "A_1".to_string(),
                            description: "desc".to_string(),
                            action_items: vec![Action::location(Coordinates::new(10, 20))],
                        },
                        Task {
                            id: "A_1".to_string(),
                            description: "desc".to_string(),
                            action_items: vec![Action::jump(Coordinates::new(30, 20))],
                        },
                    ],
                },
                create_simple_step("B"),
                create_simple_step("C"),
            ])
            .build();
        let err = QuestValidationError::NotUniqueIDForStepTask("A".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        // Should not be valid because of Tasks::None
        let quest = QuestBuilder::new()
            .with_connections(vec![Connection::new("A", "B"), Connection::new("B", "C")])
            .with_steps(vec![
                create_simple_step("A"),
                create_empty_step("B"),
                create_simple_step("C"),
            ])
            .build();
        let err = QuestValidationError::MissingTasksForStep("B".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        let quest = QuestBuilder::new()
            .with_connections(vec![Connection::new("A", "B"), Connection::new("B", "C")])
            .with_steps(vec![
                Step {
                    id: "A".to_string(),
                    description: "".to_string(),
                    tasks: vec![Task {
                        id: "A_1".to_string(),
                        description: "desc".to_string(),
                        action_items: vec![Action::location(Coordinates::new(10, 20))],
                    }],
                },
                create_empty_step("B"),
                create_simple_step("C"),
            ])
            .build();
        let err = QuestValidationError::MissingDescriptionForStep("A".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);

        let quest = QuestBuilder::new()
            .with_connections(vec![Connection::new("A", "B"), Connection::new("B", "C")])
            .with_steps(vec![
                Step {
                    id: "A".to_string(),
                    description: "desc".to_string(),
                    tasks: vec![
                        Task {
                            id: "A_1".to_string(),
                            description: "".to_string(),
                            action_items: vec![Action::location(Coordinates::new(10, 20))],
                        },
                        Task {
                            id: "A_2".to_string(),
                            description: "desc".to_string(),
                            action_items: vec![Action::location(Coordinates::new(30, 20))],
                        },
                    ],
                },
                create_empty_step("B"),
                create_simple_step("C"),
            ])
            .build();
        let err = QuestValidationError::MissingDescriptionForTask("A_1".to_string());
        assert_eq!(quest.is_valid().unwrap_err(), err);
    }
}
