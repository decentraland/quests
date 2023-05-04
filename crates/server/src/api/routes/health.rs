use actix_web::{get, web::ServiceConfig, HttpResponse};

#[utoipa::path(
    responses(
        (status = 200, description = "Service is live")
    )
)]
#[get("/health/live")]
async fn live() -> HttpResponse {
    HttpResponse::Ok().json("alive")
}

pub fn services(config: &mut ServiceConfig) {
    config.service(live);
}
