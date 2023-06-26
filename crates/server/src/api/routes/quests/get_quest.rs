use crate::domain::quests::QuestError;

use super::ProtectedQuest;
use actix_web::{get, web, HttpMessage, HttpRequest, HttpResponse};
use dcl_crypto::Address;
use quests_db::Database;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct GetQuestResponse {
    pub quest: ProtectedQuest,
}

#[utoipa::path(
    params(
        ("quest_id" = String, description = "Quest ID")
    ),
    responses(
        (status = 200, description = "Quest definition", body = GetQuestResponse),
        (status = 404, description = "Quest not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/quests/{quest_id}")]
pub async fn get_quest(
    req: HttpRequest,
    data: web::Data<Database>,
    quest_id: web::Path<String>,
) -> HttpResponse {
    let db = data.into_inner();

    let user = {
        let extensions = req.extensions();
        if let Some(address) = extensions.get::<Option<Address>>() {
            *address
        } else {
            None
        }
    };

    let quest_id = quest_id.into_inner();

    match quests_system::quests::get_quest(db, &quest_id).await {
        Ok(quest) => HttpResponse::Ok().json(GetQuestResponse {
            quest: ProtectedQuest {
                id: quest_id,
                name: quest.name,
                description: quest.description,
                image_url: quest.image_url,
                definition: if let Some(address) = user {
                    if address
                        .to_string()
                        .eq_ignore_ascii_case(&quest.creator_address)
                    {
                        quest.definition
                    } else {
                        None
                    }
                } else {
                    None
                },
            },
        }),
        Err(err) => {
            let err: QuestError = err.into();
            HttpResponse::from_error(err)
        }
    }
}
