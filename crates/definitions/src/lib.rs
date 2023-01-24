pub mod quests {
    use serde::{Deserialize, Serialize};

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
        pub tasks: Vec<Task>,
        /// Allow hooks on every completed step
        pub on_complete_hook: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Task {
        pub title: String,
        pub description: String,
        /// Required actions to complete the task
        pub action_items: Vec<Action>,
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
}
