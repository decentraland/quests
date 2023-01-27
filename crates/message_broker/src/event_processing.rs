use std::sync::Arc;

use quests_db::core::definitions::QuestsDatabase;
use quests_definitions::quests::Event;

use crate::{events_queue::EventsQueue, quests_channel::QuestsChannel};

pub async fn process_event(
    event: Event,
    quests_channel: Arc<impl QuestsChannel>,
    database: Arc<impl QuestsDatabase>,
    events_queue: Arc<impl EventsQueue>,
) {
    todo!()
}
