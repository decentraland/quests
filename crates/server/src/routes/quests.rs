use actix_web::{delete, get, post, put, web::ServiceConfig, HttpResponse};

#[get("/quests")]
async fn get_quests() -> HttpResponse {
    todo!()
}

#[post("/quests")]
async fn create_quest() -> HttpResponse {
    todo!()
}

#[put("/quests/{quest_id}")]
async fn update_quest() -> HttpResponse {
    todo!()
}

#[delete("/quests/{quest_id}")]
async fn delete_quest() -> HttpResponse {
    todo!()
}

#[get("/quests/{quest_id}")]
async fn get_quest() -> HttpResponse {
    todo!()
}

#[get("/quests/{quest_id}/stats")]
async fn get_quest_stats() -> HttpResponse {
    todo!()
}

pub fn services(config: &mut ServiceConfig) {
    config
        .service(get_quests)
        .service(create_quest)
        .service(update_quest)
        .service(delete_quest)
        .service(get_quest)
        .service(get_quest_stats);
}
