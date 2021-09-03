use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;
use thiserror::Error;

use crate::models::model_errors::ModelError;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("The request contains bad data (missing parameters or incorrect call)")]
    BadRequest(String),
    #[error("The user requested was not found")]
    UserNotFound(String),
    #[error("You do not have the authorization to access this resource")]
    Unauthorized(String),
    #[error("Internal Server Error")]
    InternalServerError(String)
}

#[derive(Serialize)]
struct ErrorResponse {
    status_code: u16,
    error: String,
    // message: String,
    error_code: String
}

impl ApiError {
    pub fn name(&self) -> String {
        match self {
            Self::BadRequest(_) => "Bad Request".to_string(),
            Self::UserNotFound(_) => "User Not Found".to_string(),
            Self::Unauthorized(_) => "Unauthorized".to_string(),
            Self::InternalServerError(_) => "Internal Server Error".to_string(),
        }
    }

    #[allow(irrefutable_let_patterns)] //This is temporarily here, it may happen that there are API errors without a code
    pub fn error_code(&self) -> String {
        match self {
            Self::BadRequest(err_code)
                | Self::UserNotFound(err_code)
                | Self::Unauthorized(err_code)
                | Self::InternalServerError(err_code) 
                => err_code.to_string(),
        }
    }
}

impl From<ModelError> for ApiError {
    fn from(e: ModelError) -> Self {
        match e {
            ModelError::AlreadyInAnotherGame
                | ModelError::AlreadyInRequestedGame 
                | ModelError::NotInGame 
                | ModelError::GameNotStarted 
                | ModelError::NoCurrentTarget 
                | ModelError::GameNotFound 
                | ModelError::AlreadyRegistered
                => Self::BadRequest(e.error_code()),
            ModelError::DatabaseError
                | ModelError::UnknownError(_)
                => Self::InternalServerError(e.error_code())
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::UserNotFound(_) => StatusCode::NOT_FOUND,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_code = self.error_code();
        let response = ErrorResponse {
            status_code: status_code.as_u16(),
            error: self.name(),
            // message: self.to_string(),
            error_code
        };
        HttpResponse::build(status_code).json(response)
    }
}
