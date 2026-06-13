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
    InternalError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Database(e) => {
                tracing::error!("DB error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "데이터베이스 오류".to_string())
            }
            ApiError::NotFound => (StatusCode::NOT_FOUND, "찾을 수 없습니다".to_string()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "인증이 필요합니다".to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "권한이 없습니다".to_string()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Internal(msg) | ApiError::InternalError(msg) => {
                tracing::error!("Internal error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, "서버 오류".to_string())
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        ApiError::Database(e)
    }
}
