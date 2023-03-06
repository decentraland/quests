use actix_web::{get, web::ServiceConfig, HttpResponse};

#[utoipa::path(
    responses(
        (status = 200, description = "Service is live")
    )
)]
#[get("/live")]
async fn live() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn services(config: &mut ServiceConfig) {
    config.service(live);
}
