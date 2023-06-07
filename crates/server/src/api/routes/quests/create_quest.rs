use std::sync::Arc;
use actix_web::{post, web, HttpResponse, HttpRequest};
use derive_more::Deref;
use quests_db::{core::definitions::{QuestsDatabase, CreateQuest, QuestReward}, Database};
use quests_protocol::definitions::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::{
    api::routes::{errors::CommonError, quests::get_user_address_from_request},
    domain::{quests::QuestError, types::ToCreateQuest},
};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateQuestResponse {
    pub id: String,
}

#[derive(Deserialize, Serialize, ToSchema, Deref, Debug)]
pub struct CreateQuestRequest {
    pub name: String,
    pub description: String,
    #[deref]
    pub definition: QuestDefinition,
    pub reward: Option<QuestReward>,
}

#[utoipa::path(
    request_body = CreateQuestRequest, 
    responses(
        (status = 201, description = "Quest created", body = CreateQuestResponse),
        (status = 400, description = "Bad Request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[post("/quests")]
pub async fn create_quest(
    req: HttpRequest,
    data: web::Data<Database>,
    quest: web::Json<CreateQuestRequest>,
) -> HttpResponse {
    let db = data.into_inner();

    let user = match get_user_address_from_request(&req) {
        Ok(address) => address,
        Err(bad_request_response) => return bad_request_response,
    };

    match create_quest_controller(db, &quest, &user).await {
        Ok(quest_id) => HttpResponse::Created().json(CreateQuestResponse { id: quest_id }),
        Err(error) => HttpResponse::from_error(error),
    }
}

async fn create_quest_controller<DB: QuestsDatabase>(
    db: Arc<DB>,
    create_quest_req: &CreateQuestRequest,
    creator_address: &str
) -> Result<String, QuestError> {
    create_quest_req
        .is_valid()
        .map_err(|error| QuestError::QuestValidation(error.to_string()))?;

    let quest = create_quest_req.to_create_quest()?;
    let id = db.create_quest(&quest, creator_address)
        .await
        .map_err(|_| QuestError::CommonError(CommonError::Unknown))?;

    if let Some(reward) = &create_quest_req.reward {
        db.add_reward_to_quest(&id, reward)
            .await
            .map_err(|_| QuestError::CommonError(CommonError::Unknown))?;
    }

    Ok(id)
}

impl ToCreateQuest for CreateQuestRequest {
    fn to_create_quest(&self) -> Result<CreateQuest, QuestError> {
        let CreateQuestRequest {
            name,
            description,
            definition,
            ..
        } = self;

        Ok(CreateQuest {
            name,
            description,
            definition: definition.encode_to_vec(),
        })
    }
}
