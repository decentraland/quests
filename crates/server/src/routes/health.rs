use actix_web::{get, web::ServiceConfig, HttpResponse};

#[get("/live")]
async fn live() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn services(config: &mut ServiceConfig) {
    config.service(live);
}
