use quests_definitions::quests::Event;
use quests_message_broker::events_queue::{EventsQueue, EventsQueueResult};
use std::sync::Arc;

pub async fn add_event_controller(
    events_queue: Arc<impl EventsQueue>,
    event: Event,
) -> EventsQueueResult<usize> {
    events_queue.push(&event).await
}
