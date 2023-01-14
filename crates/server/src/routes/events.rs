use actix_web::{web::ServiceConfig, put, HttpResponse};

#[put("/events")]
async fn add_event() -> HttpResponse {
    todo!()
}

pub fn services(config: &mut ServiceConfig) {
    config.service(add_event);
}
