use actix_web::{get, HttpResponse};

#[get("/quests/{quest_id}/stats")]
pub async fn get_quest_stats() -> HttpResponse {
    todo!()
}
