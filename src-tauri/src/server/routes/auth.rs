use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{error::ApiError, server::state::AppState};

const SESSION_COOKIE: &str = "cc_session";
const SESSION_MAX_AGE_SECS: i64 = 12 * 60 * 60; // 12시간

fn parse_session_token(headers: &HeaderMap) -> Option<String> {
    let cookie_str = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in cookie_str.split(';') {
        let part = part.trim();
        if let Some(val) = part.strip_prefix(&format!("{SESSION_COOKIE}=")) {
            return Some(val.to_string());
        }
    }
    None
}

fn set_session_cookie(token: &str) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "{SESSION_COOKIE}={token}; HttpOnly; Path=/; SameSite=Lax; Max-Age={SESSION_MAX_AGE_SECS}"
    ))
    .unwrap()
}

fn clear_session_cookie() -> HeaderValue {
    HeaderValue::from_str(&format!(
        "{SESSION_COOKIE}=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0"
    ))
    .unwrap()
}

#[derive(Deserialize)]
pub struct TeacherLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct MeResponse {
    pub id: i64,
    pub username: String,
    pub name: String,
    pub role: String,
}

/// POST /api/auth/login/teacher
pub async fn teacher_login(
    State(state): State<AppState>,
    req_headers: HeaderMap,
    Json(body): Json<TeacherLoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let row = sqlx::query_as::<_, (i64, String, String, String)>(
        "SELECT id, password_hash, name, role FROM teachers WHERE username = ?",
    )
    .bind(&body.username)
    .fetch_optional(&state.db)
    .await?;

    let (teacher_id, password_hash, name, role) = row.ok_or(ApiError::Unauthorized)?;

    let parsed_hash =
        PasswordHash::new(&password_hash).map_err(|e| ApiError::Internal(e.to_string()))?;

    Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .map_err(|_| ApiError::Unauthorized)?;

    let token = Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(SESSION_MAX_AGE_SECS))
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    sqlx::query("INSERT INTO auth_tokens (teacher_id, token, expires_at) VALUES (?, ?, ?)")
        .bind(teacher_id)
        .bind(&token)
        .bind(&expires_at)
        .execute(&state.db)
        .await?;

    let _ = req_headers; // used for consistency with other handlers

    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, set_session_cookie(&token));

    Ok((
        StatusCode::OK,
        headers,
        Json(json!({
            "ok": true,
            "user": {
                "id": teacher_id,
                "username": body.username,
                "name": name,
                "role": role,
            }
        })),
    ))
}

/// POST /api/auth/logout
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(token) = parse_session_token(&headers) {
        sqlx::query("DELETE FROM auth_tokens WHERE token = ?")
            .bind(&token)
            .execute(&state.db)
            .await?;
    }

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(header::SET_COOKIE, clear_session_cookie());

    Ok((StatusCode::OK, resp_headers, Json(json!({ "ok": true }))))
}

/// GET /api/auth/me
pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<MeResponse>, ApiError> {
    let token = parse_session_token(&headers).ok_or(ApiError::Unauthorized)?;

    let row = sqlx::query_as::<_, (i64, String, String, String)>(
        r#"SELECT t.id, t.username, t.name, t.role
           FROM auth_tokens a
           JOIN teachers t ON t.id = a.teacher_id
           WHERE a.token = ? AND a.expires_at > datetime('now')"#,
    )
    .bind(&token)
    .fetch_optional(&state.db)
    .await?;

    let (id, username, name, role) = row.ok_or(ApiError::Unauthorized)?;

    Ok(Json(MeResponse {
        id,
        username,
        name,
        role,
    }))
}

/// GET /api/auth/school-name
pub async fn school_name(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let name = sqlx::query_scalar::<_, String>("SELECT school_name FROM settings WHERE id = 1")
        .fetch_optional(&state.db)
        .await?
        .unwrap_or_default();

    Ok(Json(json!({ "school_name": name })))
}
