use quests_definitions::quests::Event;
use quests_message_broker::events_queue::{EventsQueue, EventsQueueResult};
use std::sync::Arc;

pub async fn add_event_controller(
    events_queue: Arc<impl EventsQueue<Event>>,
    event: Event,
) -> EventsQueueResult<usize> {
    if event.action.is_some() {
        events_queue.push(&event).await
    } else {
        Err("No Action".to_string())
    }
}
