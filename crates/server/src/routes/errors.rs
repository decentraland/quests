use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
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

#[derive(Error, Debug)]
pub enum QuestError {
    #[error("{0}")]
    CommonError(CommonError),
    #[error("Quest Definition issue")]
    StepsDeserialization,
    #[error("Quest Validation Error: {0}")]
    QuestValidation(String),
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
