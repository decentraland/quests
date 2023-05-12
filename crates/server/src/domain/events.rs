use quests_message_broker::messages_queue::MessagesQueue;
use quests_protocol::definitions::*;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AddEventError {
    #[error("No given action")]
    NoAction,
    #[error("Push to the queue failed")]
    PushFailed,
}

pub async fn add_event_controller(
    events_queue: Arc<impl MessagesQueue<Event>>,
    event: EventRequest,
) -> Result<Uuid, AddEventError> {
    if let Some(action) = event.action {
        let id = Uuid::new_v4();
        let event = Event {
            id: id.to_string(),
            address: event.address,
            action: Some(action),
        };
        if events_queue.push(&event).await.is_ok() {
            Ok(id)
        } else {
            Err(AddEventError::PushFailed)
        }
    } else {
        Err(AddEventError::NoAction)
    }
}
