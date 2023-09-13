use crate::domain::quests::QuestError;
use actix_web::{error::JsonPayloadError, http::StatusCode, web, HttpResponse, ResponseError};
use quests_db::core::errors::DBError;
use quests_protocol::definitions::*;
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
    #[error("Bad Request: the given ID is not valid")]
    NotUUID,
}

impl ResponseError for CommonError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::NotUUID => StatusCode::BAD_REQUEST,
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
            Self::DeserializationError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::CommonError(base) => base.status_code(),
            Self::QuestValidation(_) | Self::QuestAlreadyStarted | Self::QuestAlreadyCompleted => {
                StatusCode::BAD_REQUEST
            }
            Self::NotInstanceOwner => StatusCode::FORBIDDEN,
            Self::NotFoundOrInactive => StatusCode::NOT_FOUND,
            Self::NotQuestCreator => StatusCode::FORBIDDEN,
            Self::QuestHasNoReward => StatusCode::NOT_FOUND,
            Self::QuestNotActivable => StatusCode::BAD_REQUEST,
            Self::QuestIsNotUpdatable => StatusCode::BAD_REQUEST,
            Self::QuestIsCurrentlyDeactivated => StatusCode::BAD_REQUEST,
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

pub fn json_extractor_config() -> web::JsonConfig {
    web::JsonConfig::default().error_handler(|err, _| match err {
        JsonPayloadError::Deserialize(des_err) => {
            let err_string = des_err.to_string();
            CommonError::BadRequest(err_string).into()
        }
        _ => CommonError::BadRequest(err.to_string()).into(),
    })
}

pub fn path_extractor_config() -> web::PathConfig {
    web::PathConfig::default()
        .error_handler(|err, _| CommonError::BadRequest(err.to_string()).into())
}

impl From<ProtocolDecodeError> for QuestError {
    fn from(_value: ProtocolDecodeError) -> Self {
        QuestError::DeserializationError
    }
}

impl From<DBError> for QuestError {
    fn from(error: DBError) -> Self {
        match error {
            DBError::NotUUID => QuestError::CommonError(CommonError::NotUUID),
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
