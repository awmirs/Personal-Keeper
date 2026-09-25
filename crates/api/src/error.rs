use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use domain::error::CoreError;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ApiError {
    #[error(transparent)]
    Core(#[from] CoreError),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Core(err) => match err {
                CoreError::NotFound(_) => StatusCode::NOT_FOUND,
                CoreError::Validation(_) => StatusCode::BAD_REQUEST,
                CoreError::Crypto(_) => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(serde_json::json!({
            "error": self.to_string()
        }))
    }
}
