use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    Database(sqlx::Error),
    NotFound,
    Unauthorized,
    Forbidden,
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match self {
            ApiError::Database(e) => {
                tracing::error!("DB error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "ERR_DB".to_string())
            }
            ApiError::NotFound => (StatusCode::NOT_FOUND, "ERR_NOT_FOUND".to_string()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "ERR_UNAUTHORIZED".to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "ERR_FORBIDDEN".to_string()),
            ApiError::BadRequest(code) => (StatusCode::BAD_REQUEST, code),
            ApiError::Internal(msg) => {
                tracing::error!("Internal error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, "ERR_INTERNAL".to_string())
            }
        };
        (status, Json(json!({ "error": code }))).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        ApiError::Database(e)
    }
}
