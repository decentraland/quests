use quests_definitions::quests::{Action, Coordinates, Event};
use quests_message_broker::events_queue::{EventsQueue, RedisEventsQueue};

mod common;
use common::redis::build_redis;

const ADDRESS: &str = "0xA";
const TIMESTAMP_0: usize = 0;

fn build_location_event(coordinates: Coordinates) -> Event {
    Event {
        address: ADDRESS.to_string(),
        timestamp: TIMESTAMP_0,
        action: Action::Location { coordinates },
    }
}
#[tokio::test]
async fn can_send_event_to_the_queue() {
    let redis = build_redis(10).await;
    let events_queue = RedisEventsQueue::new(redis);

    let event = build_location_event(Coordinates(0, 0));

    let result = events_queue.push(&event).await;
    assert!(result.is_ok(), "should be able to send events");
    if let Err(reason) = result {
        println!("{}", reason);
    }
}

#[tokio::test]
async fn can_send_multiple_events_to_the_queue() {
    let redis = build_redis(11).await;
    let events_queue = RedisEventsQueue::new(redis);

    let event = build_location_event(Coordinates(0, 0));
    let result = events_queue.push(&event).await;
    assert!(result.is_ok(), "should be able to push events");

    let event = build_location_event(Coordinates(0, 1));
    let result = events_queue.push(&event).await;
    assert!(result.is_ok(), "should be able to push events");
}

#[tokio::test]
async fn can_receive_event_from_the_queue() {
    let redis = build_redis(12).await;
    let events_queue = RedisEventsQueue::new(redis);

    let event = build_location_event(Coordinates(0, 0));
    let result = events_queue.push(&event).await;
    assert!(result.is_ok(), "should be able to push events");

    let result = events_queue.pop().await;
    assert!(result.is_ok(), "should be able to pop event");
    let received_event = result.unwrap();
    assert_eq!(event, received_event);
}

#[tokio::test]
async fn can_receive_multiple_events_from_the_queue() {
    let redis = build_redis(13).await;
    let events_queue = RedisEventsQueue::new(redis);

    let first_event = build_location_event(Coordinates(0, 0));
    let result = events_queue.push(&first_event).await;
    assert!(result.is_ok(), "should be able to push events");

    let second_event = build_location_event(Coordinates(0, 1));
    let result = events_queue.push(&second_event).await;
    assert!(result.is_ok(), "should be able to push events");

    let first_pop = events_queue.pop().await;
    let second_pop = events_queue.pop().await;

    assert!(first_pop.is_ok(), "should be able to pop event");
    assert!(second_pop.is_ok(), "should be able to pop event");

    let received_event = first_pop.unwrap();
    assert_eq!(first_event, received_event);
    assert_ne!(second_event, received_event);

    let received_event = second_pop.unwrap();
    assert_eq!(second_event, received_event);
}
