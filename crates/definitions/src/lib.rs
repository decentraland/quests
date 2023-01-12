pub mod quests {

    pub struct Quest {
        pub name: String,
        pub description: String,
        pub steps: Vec<Step>,
    }

    pub struct Step {
        pub id: String,
        pub description: String,
        pub tasks: Vec<Task>,
        /// Allow hooks on every completed step
        pub on_complete_hook: Option<String>,
    }

    pub struct Task {
        pub title: String,
        pub description: String,
        /// Required actions to complete the task
        pub action_items: Vec<Action>,
    }

    pub struct Event {
        pub address: String,
        pub timestamp: usize,
        pub action: Action,
    }

    pub struct Coordinates(usize, usize);

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
