use actix_web::{
    error::ErrorBadRequest,
    put,
    web::{self, ServiceConfig},
    HttpResponse,
};
use quests_definitions::quests::Event;
use quests_message_broker::events_queue::{EventsQueue, RedisEventsQueue};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct AddEventResponse(String);

#[utoipa::path(
    request_body = Event,
    responses(
        (status = 202, description = "Event accepted", body = AddEventResponse),
        (status = 400, description = "Bad Request")
    )
)]
#[put("/events")]
async fn add_event(data: web::Data<RedisEventsQueue>, event: web::Json<Event>) -> HttpResponse {
    let events_queue = data.into_inner();
    match events_queue.push(&event.into_inner()).await {
        Ok(_) => HttpResponse::Accepted().json(AddEventResponse("Event Accepted".to_string())),
        Err(err) => HttpResponse::from_error(ErrorBadRequest(err)),
    }
}

pub fn services(config: &mut ServiceConfig) {
    config.service(add_event);
}
