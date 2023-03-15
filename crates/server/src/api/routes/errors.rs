use crate::logic::quests::QuestError;
use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use quests_db::core::errors::DBError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
}

#[derive(Error, Debug)]
pub enum CommonError {
    #[error("Unknown Internal Error")]
    Unknown,
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Not Found")]
    NotFound,
    // TODO: Here we should add errors like "Bad Request", "Forbidden", "Unathorized", "Not Found"
}

impl ResponseError for CommonError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
        };
        HttpResponse::build(status_code).json(error_response)
    }
}

impl ResponseError for QuestError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::StepsDeserialization => StatusCode::NOT_FOUND,
            Self::CommonError(base) => base.status_code(),
            Self::QuestValidation(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
        };
        HttpResponse::build(status_code).json(error_response)
    }
}

pub fn query_extractor_config() -> web::QueryConfig {
    web::QueryConfig::default()
        .error_handler(|err, _| CommonError::BadRequest(err.to_string()).into())
}

impl From<bincode::Error> for QuestError {
    fn from(_value: bincode::Error) -> Self {
        QuestError::StepsDeserialization
    }
}

impl From<DBError> for QuestError {
    fn from(error: DBError) -> Self {
        match error {
            DBError::NotUUID => QuestError::CommonError(CommonError::BadRequest(
                "the ID given is not a valid".to_string(),
            )),
            DBError::RowNotFound => QuestError::CommonError(CommonError::NotFound),
            _ => QuestError::CommonError(CommonError::Unknown),
        }
    }
}

impl From<DBError> for CommonError {
    fn from(error: DBError) -> Self {
        match error {
            DBError::NotUUID => CommonError::BadRequest("the ID given is not a valid".to_string()),
            DBError::RowNotFound => CommonError::NotFound,
            _ => CommonError::Unknown,
        }
    }
}
