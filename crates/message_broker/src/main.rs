use quests_message_broker::{start_event_processing, EventProcessingResult};

#[tokio::main]
async fn main() -> EventProcessingResult<()> {
    // Listen to the new events and process them in separated tasks
    start_event_processing().await
}
