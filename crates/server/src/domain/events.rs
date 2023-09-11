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
    user_address: &str,
    event: EventRequest,
) -> Result<Uuid, AddEventError> {
    if let Some(action) = event.action {
        let id = Uuid::new_v4();
        let event = Event {
            id: id.to_string(),
            address: user_address.to_string(),
            action: Some(action),
        };
        match events_queue.push(&event).await {
            Ok(queue_size) => {
                log::debug!("Pushed event to the queue, queue size: {queue_size}");
                Ok(id)
            }
            Err(e) => {
                log::error!("Failed to push event to the queue {e}");
                Err(AddEventError::PushFailed)
            }
        }
    } else {
        Err(AddEventError::NoAction)
    }
}
