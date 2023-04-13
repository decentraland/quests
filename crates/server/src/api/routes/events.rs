use crate::domain::events::add_event_controller;
use actix_web::{
    error::ErrorBadRequest,
    put,
    web::{self, ServiceConfig},
    HttpResponse,
};
use quests_message_broker::messages_queue::RedisMessagesQueue;
use quests_protocol::quests::Event;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct AddEventResponse {
    message: String,
}

#[utoipa::path(
    request_body = Event,
    responses(
        (status = 202, description = "Event accepted", body = AddEventResponse),
        (status = 400, description = "Bad Request")
    )
)]
#[put("/events")]
async fn add_event(data: web::Data<RedisMessagesQueue>, event: web::Json<Event>) -> HttpResponse {
    let events_queue = data.into_inner();
    match add_event_controller(events_queue, event.into_inner()).await {
        Ok(_) => HttpResponse::Accepted().json(AddEventResponse {
            message: "Event Accepted".to_string(),
        }),
        Err(err) => HttpResponse::from_error(ErrorBadRequest(err)),
    }
}

pub fn services(config: &mut ServiceConfig) {
    config.service(add_event);
}
