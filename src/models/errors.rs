use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("The request is missing required data")]
    BadRequest,
    #[error("The user requested was not found")]
    UserNotFound,
    #[error("You do not have the authorization to access this resource")]
    Unauthorized,
    #[error("Internal Server Error")]
    InternalServerError(#[from] color_eyre::Report),
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}

impl ApiError {
    pub fn name(&self) -> String {
        match self {
            Self::BadRequest => "Bad Request".to_string(),
            Self::UserNotFound => "User Not Found".to_string(),
            Self::Unauthorized => "Unauthorized".to_string(),
            Self::InternalServerError(_) => "Internal Server Error".to_string(),
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::UserNotFound => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
            error: self.name(),
        };
        HttpResponse::build(status_code).json(response)
    }
}
