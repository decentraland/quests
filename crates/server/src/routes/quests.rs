use actix_web::{
    delete, get, post, put,
    web::{self, ServiceConfig},
    HttpResponse,
};
use quests_db_core::{CreateQuest, QuestsDatabase, StoredQuest, UpdateQuest};
use quests_definitions::quests::Quest;
use serde::Deserialize;

use crate::components::AppComponents;

use super::CommonError;

#[derive(Deserialize)]
struct GetQuestsQuery {
    offset: u64,
    limit: u64,
}

#[get("/quests")]
async fn get_quests(
    data: web::Data<AppComponents>,
    query: web::Query<GetQuestsQuery>,
) -> HttpResponse {
    match get_quests_controller(&data.database, query.offset, query.limit).await {
        Ok(quests) => HttpResponse::Ok().json(quests),
        Err(error) => {
            log::error!("> /quests > Error while querying DB {error:?}");
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn get_quests_controller<DB: QuestsDatabase>(
    db: &DB,
    offset: u64,
    limit: u64,
) -> Result<Vec<StoredQuest>, CommonError> {
    db.get_quests(offset, limit)
        .await
        .map_err(|_| CommonError::Unknown)
}

#[post("/quests")]
async fn create_quest(data: web::Data<AppComponents>, quest: web::Json<Quest>) -> HttpResponse {
    match create_quest_controller(&data.database, quest.0).await {
        Ok(quest) => HttpResponse::Created().json(quest),
        Err(error) => HttpResponse::from_error(error),
    }
}

async fn create_quest_controller<DB: QuestsDatabase>(
    db: &DB,
    quest: Quest,
) -> Result<Quest, CommonError> {
    let quest_creation = CreateQuest {
        name: &quest.name,
        description: &quest.description,
        definition: bincode::serialize(&quest.steps).unwrap(),
    };
    match db.create_quest(&quest_creation).await {
        Ok(()) => Ok(quest),
        Err(_) => Err(CommonError::Unknown),
    }
}

#[put("/quests/{quest_id}")]
async fn update_quest(
    data: web::Data<AppComponents>,
    quest_id: web::Path<u32>,
    quest_update: web::Json<Quest>,
) -> HttpResponse {
    match update_quest_controller(&data.database, quest_id.into_inner(), quest_update.0).await {
        Ok(quest) => HttpResponse::Ok().json(quest),
        Err(error) => HttpResponse::from_error(error),
    }
}

async fn update_quest_controller<DB: QuestsDatabase>(
    db: &DB,
    id: u32,
    quest: Quest,
) -> Result<Quest, CommonError> {
    let update = UpdateQuest {
        name: &quest.name,
        description: &quest.description,
        definition: bincode::serialize(&quest.steps).unwrap(),
    };
    match db.update_quest(format!("{}", id).as_str(), &update).await {
        Ok(_) => Ok(quest),
        Err(_) => Err(CommonError::Unknown),
    }
}

#[delete("/quests/{quest_id}")]
async fn delete_quest(data: web::Data<AppComponents>, quest_id: web::Path<u32>) -> HttpResponse {
    match delete_quest_controller(&data.database, quest_id.into_inner()).await {
        Ok(()) => HttpResponse::Accepted().finish(),
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn delete_quest_controller<DB: QuestsDatabase>(db: &DB, id: u32) -> Result<(), CommonError> {
    match db.delete_quest(format!("{}", id).as_str()).await {
        Ok(_) => Ok(()),
        Err(_) => Err(CommonError::Unknown),
    }
}

#[get("/quests/{quest_id}")]
async fn get_quest(data: web::Data<AppComponents>, quest_id: web::Path<u32>) -> HttpResponse {
    match get_quest_controller(&data.database, quest_id.into_inner()).await {
        Ok(quest) => HttpResponse::Ok().json(quest),
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn get_quest_controller<DB: QuestsDatabase>(db: &DB, id: u32) -> Result<Quest, CommonError> {
    match db.get_quest(format!("{}", id).as_str()).await {
        Ok(stored_quest) => {
            let quest = Quest {
                name: stored_quest.name,
                description: stored_quest.description,
                steps: bincode::deserialize(&stored_quest.definition).unwrap(),
            };
            Ok(quest)
        }
        Err(_) => Err(CommonError::Unknown),
    }
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
