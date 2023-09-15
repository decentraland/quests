use crate::{
    api::routes::quests::get_user_address_from_request,
    domain::{quests::QuestError, types::ToQuest},
};

use actix_web::{get, web, HttpRequest, HttpResponse};
use quests_db::{core::definitions::QuestsDatabase, Database};
use quests_protocol::definitions::Quest;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams)]
pub struct GetQuestsQuery {
    offset: Option<i64>,
    limit: Option<i64>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GetCreatorQuestsResponse {
    pub quests: Vec<Quest>,
}

/// Get quests by creator id
/// Returns a list of quests created by the user
#[utoipa::path(
    params(
        ("user_address" = String, description = "Creator's Ethereum Address")
    ),
    responses(
        (status = 200, description = "Quest definition", body = GetQuestResponse),
        (status = 404, description = "Quest not found"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/creators/{user_address}/quests")]
pub async fn get_quests_by_creator_id(
    req: HttpRequest,
    data: web::Data<Database>,
    user_address: web::Path<String>,
    query: web::Query<GetQuestsQuery>,
) -> HttpResponse {
    let db = data.into_inner();

    let user = get_user_address_from_request(&req).ok();

    let is_owner = if let Some(address) = user {
        address.eq_ignore_ascii_case(&user_address)
    } else {
        false
    };

    match db
        .get_quests_by_creator_id(
            &user_address.to_lowercase(),
            query.offset.unwrap_or(0),
            query.limit.unwrap_or(50),
        )
        .await
    {
        Ok(stored_quests) => {
            let mut quests = vec![];
            for stored_quest in stored_quests {
                match stored_quest.to_quest(is_owner) {
                    Ok(quest) => quests.push(quest),
                    Err(err) => return HttpResponse::from_error(err),
                }
            }
            HttpResponse::Ok().json(GetCreatorQuestsResponse { quests })
        }
        Err(err) => {
            log::error!("Error getting quests: {:?}", err);
            let err: QuestError = err.into();
            HttpResponse::from_error(err)
        }
    }
}
