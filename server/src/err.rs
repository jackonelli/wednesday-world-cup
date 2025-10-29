use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use thiserror::Error;
use wwc_core::error::WwcError;

/// Error handling
#[derive(Error, Debug)]
pub(crate) enum AppError {
    #[error("Database error: {0}")]
    Db(#[from] wwc_db::DbError),
    #[error("WWC core error: {0}")]
    Wwc(#[from] WwcError),
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("{0}")]
    Generic(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            error: String,
        }

        let (status, error_message) = match self {
            AppError::Db(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Wwc(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::Sqlx(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Generic(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        let error_response = ErrorResponse {
            error: error_message,
        };

        (status, Json(error_response)).into_response()
    }
}
