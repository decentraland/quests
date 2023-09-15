use crate::{api::routes::quests::get_user_address_from_request, domain::quests::QuestError};

use actix_web::{get, web, HttpRequest, HttpResponse};
use quests_db::Database;
use quests_protocol::definitions::Quest;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct GetQuestResponse {
    pub quest: Quest,
}

/// Get a quest.
///
/// Returns the quest definition if the user is the creator of the quest (authentication required)
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

    let user = get_user_address_from_request(&req).ok();

    let quest_id = quest_id.into_inner();

    match quests_system::quests::get_quest(db, &quest_id).await {
        Ok(quest) => HttpResponse::Ok().json(GetQuestResponse {
            quest: Quest {
                definition: if let Some(address) = user {
                    if address.eq_ignore_ascii_case(&quest.creator_address) {
                        quest.definition
                    } else {
                        None
                    }
                } else {
                    None
                },
                ..quest
            },
        }),
        Err(err) => {
            let err: QuestError = err.into();
            HttpResponse::from_error(err)
        }
    }
}
