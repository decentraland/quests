use quests_message_broker::messages_queue::{MessagesQueue, MessagesQueueResult};
use quests_protocol::quests::Event;
use std::sync::Arc;

pub async fn add_event_controller(
    events_queue: Arc<impl MessagesQueue<Event>>,
    event: Event,
) -> MessagesQueueResult<usize> {
    if event.action.is_some() {
        events_queue.push(&event).await
    } else {
        Err("No Action".to_string())
    }
}
