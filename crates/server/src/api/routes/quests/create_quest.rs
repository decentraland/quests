use std::sync::Arc;
use actix_web::{post, web, HttpResponse, HttpRequest};
use quests_db::{core::definitions::{QuestsDatabase, CreateQuest, QuestRewardHook, QuestRewardItem}, Database};
use quests_protocol::definitions::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::{
    api::routes::{errors::CommonError, quests::get_user_address_from_request},
    domain::{quests::QuestError, types::ToCreateQuest},
};
use super::is_url;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateQuestResponse {
    pub id: String,
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuestReward {
    pub hook: QuestRewardHook,
    pub items: Vec<QuestRewardItem>
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateQuestRequest {
    pub name: String,
    pub description: String,
    pub definition: QuestDefinition,
    pub image_url: String,
    pub reward: Option<QuestReward>,
}

impl CreateQuestRequest {
    pub fn is_valid(&self) -> Result<(), QuestError> {
        if self.name.trim().len() < 5 {
            return Err(QuestError::QuestValidation("Name should be longer".to_string()))
        }

        if self.description.trim().len() < 10 {
            return Err(QuestError::QuestValidation("Description should be longer".to_string()))
        }

        self.definition
        .is_valid()
        .map_err(|error| QuestError::QuestValidation(error.to_string()))?;

        if let Some(QuestReward { hook, items }) = &self.reward {
            if !is_url(&hook.webhook_url) {
                return Err(QuestError::QuestValidation("Webhook url is not valid".to_string()));
            }
    
            if !items.is_empty() {
                if !items.iter().all(|item| is_url(&item.image_link)) {
                    return Err(QuestError::QuestValidation("Item's image link is not valid".to_string()));
                }
        
                if !items.iter().all(|item| item.name.len() >= 3) {
                    return Err(QuestError::QuestValidation("Item name must be at least 3 characters".to_string()));
                }
            }
        } else {
            return Err(QuestError::QuestValidation("Reward items must be at least one".to_string()));
        }

        Ok(())
    }
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
        .is_valid()?; 
    
    let quest = create_quest_req.to_create_quest()?;
    let id = db.create_quest(&quest, creator_address)
        .await
        .map_err(|_| QuestError::CommonError(CommonError::Unknown))?;

    if let Some(QuestReward { hook, items }) = &create_quest_req.reward {
        db.add_reward_hook_to_quest(&id, hook)
        .await
        .map_err(|_| QuestError::CommonError(CommonError::Unknown))?;

        db.add_reward_items_to_quest(&id, items).await.map_err(|_| QuestError::CommonError(CommonError::Unknown))?;
    }

    Ok(id)
}

impl ToCreateQuest for CreateQuestRequest {
    fn to_create_quest(&self) -> Result<CreateQuest, QuestError> {
        let CreateQuestRequest {
            name,
            description,
            definition,
            image_url,
            ..
        } = self;

        Ok(CreateQuest {
            name,
            description,
            image_url,
            definition: definition.encode_to_vec(),
        })
    }
}
