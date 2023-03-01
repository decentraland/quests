use actix_web::{get, HttpResponse};

#[utoipa::path(
    responses(
        (status = 200, description = "Quest Stats"),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Quest not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/quests/{quest_id}/stats")]
pub async fn get_quest_stats() -> HttpResponse {
    todo!()
}
