use actix_web::{
    error::ErrorBadRequest,
    put,
    web::{self, ServiceConfig},
    HttpResponse,
};
use quests_definitions::quests::Event;
use quests_message_broker::events_queue::{EventsQueue, RedisEventsQueue};

#[put("/events")]
async fn add_event(data: web::Data<RedisEventsQueue>, event: web::Json<Event>) -> HttpResponse {
    let events_queue = data.into_inner();
    match events_queue.push(&event.into_inner()).await {
        Ok(_) => HttpResponse::Accepted().finish(),
        Err(err) => HttpResponse::from_error(ErrorBadRequest(err)),
    }
}

pub fn services(config: &mut ServiceConfig) {
    config.service(add_event);
}
