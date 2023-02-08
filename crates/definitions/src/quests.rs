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
    pub fn get_steps_without_to(&self) -> HashSet<StepID> {
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
    pub fn get_steps_without_from(&self) -> HashSet<StepID> {
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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QuestDefinition {
    pub steps: Vec<Step>,
    /// Connections between steps
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
}
