use std::{collections::HashMap, sync::Arc};

use actix_web::{
    delete, get, post, put,
    web::{self, ServiceConfig},
    HttpResponse,
};
use quests_db::{
    core::definitions::{CreateQuest, QuestsDatabase, StoredQuest, UpdateQuest},
    core::errors::DBError,
    Database,
};
use quests_definitions::quests::{Quest, QuestDefinition};
use serde::Deserialize;

use super::{errors::QuestError, CommonError};

#[derive(Deserialize)]
struct GetQuestsQuery {
    offset: Option<i64>,
    limit: Option<i64>,
}

#[get("/quests")]
async fn get_quests(db: web::Data<Database>, query: web::Query<GetQuestsQuery>) -> HttpResponse {
    let db = db.into_inner();
    match get_quests_controller(db, query.offset.unwrap_or(0), query.limit.unwrap_or(50)).await {
        Ok(quests) => HttpResponse::Ok().json(quests),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn get_quests_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    offset: i64,
    limit: i64,
) -> Result<Vec<StoredQuest>, CommonError> {
    db.get_quests(offset, limit)
        .await
        .map_err(|_| CommonError::Unknown)
}

#[post("/quests")]
async fn create_quest(data: web::Data<Database>, quest: web::Json<Quest>) -> HttpResponse {
    let db = data.into_inner();
    match create_quest_controller(db, quest.0).await {
        Ok(quest_id) => {
            let mut response_body = HashMap::new();
            response_body.insert("id", quest_id);
            HttpResponse::Created().json(response_body)
        }
        Err(error) => HttpResponse::from_error(error),
    }
}

async fn create_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    quest: Quest,
) -> Result<String, CommonError> {
    let quest_creation = CreateQuest {
        name: &quest.name,
        description: &quest.description,
        definition: bincode::serialize(&quest.definition).unwrap(),
    };
    match db.create_quest(&quest_creation).await {
        Ok(quest_id) => Ok(quest_id),
        Err(_) => Err(CommonError::Unknown),
    }
}

#[put("/quests/{quest_id}")]
async fn update_quest(
    data: web::Data<Database>,
    quest_id: web::Path<String>,
    quest_update: web::Json<Quest>,
) -> HttpResponse {
    let db = data.into_inner();
    let quest_id = quest_id.into_inner();
    match update_quest_controller(db, quest_id, quest_update.0).await {
        Ok(quest) => HttpResponse::Ok().json(quest),
        Err(error) => HttpResponse::from_error(error),
    }
}

async fn update_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    id: String,
    quest: Quest,
) -> Result<Quest, CommonError> {
    let update = UpdateQuest {
        name: &quest.name,
        description: &quest.description,
        definition: bincode::serialize(&quest.definition).unwrap(),
    };
    match db.update_quest(&id, &update).await {
        Ok(_) => Ok(quest),
        Err(error) => match error {
            DBError::NotUUID => Err(CommonError::BadRequest(
                "the ID given is not a valid".to_string(),
            )),
            DBError::RowNotFound => Err(CommonError::NotFound),
            _ => Err(CommonError::Unknown),
        },
    }
}

#[delete("/quests/{quest_id}")]
async fn delete_quest(data: web::Data<Database>, quest_id: web::Path<String>) -> HttpResponse {
    let db = data.into_inner();
    match delete_quest_controller(db, quest_id.into_inner()).await {
        Ok(()) => HttpResponse::Accepted().finish(),
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn delete_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    id: String,
) -> Result<(), CommonError> {
    match db.delete_quest(&id).await {
        Ok(_) => Ok(()),
        Err(error) => match error {
            DBError::NotUUID => Err(CommonError::BadRequest(
                "the ID given is not a valid".to_string(),
            )),
            DBError::RowNotFound => Err(CommonError::NotFound),
            _ => Err(CommonError::Unknown),
        },
    }
}

#[get("/quests/{quest_id}")]
async fn get_quest(data: web::Data<Database>, quest_id: web::Path<String>) -> HttpResponse {
    let db = data.into_inner();
    match get_quest_controller(db, quest_id.into_inner()).await {
        Ok(quest) => HttpResponse::Ok().json(quest),
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn get_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    id: String,
) -> Result<Quest, QuestError> {
    match db.get_quest(&id).await {
        Ok(stored_quest) => {
            let definition: QuestDefinition =
                if let Ok(definition) = bincode::deserialize(&stored_quest.definition) {
                    definition
                } else {
                    return Err(QuestError::StepsDeserialization);
                };
            let quest = Quest {
                name: stored_quest.name,
                description: stored_quest.description,
                definition,
            };
            Ok(quest)
        }
        Err(error) => match error {
            DBError::NotUUID => Err(QuestError::CommonError(CommonError::BadRequest(
                "the ID given is not a valid".to_string(),
            ))),
            DBError::RowNotFound => Err(QuestError::CommonError(CommonError::NotFound)),
            _ => Err(QuestError::CommonError(CommonError::Unknown)),
        },
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
