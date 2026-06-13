use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{error::ApiError, server::state::AppState};

const SESSION_COOKIE: &str = "cc_session";
const STUDENT_SESSION_COOKIE: &str = "cc_student";
const SESSION_MAX_AGE_SECS: i64 = 12 * 60 * 60; // 12시간

fn parse_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookie_str = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in cookie_str.split(';') {
        let part = part.trim();
        if let Some(val) = part.strip_prefix(&format!("{name}=")) {
            return Some(val.to_string());
        }
    }
    None
}

fn parse_session_token(headers: &HeaderMap) -> Option<String> {
    parse_cookie(headers, SESSION_COOKIE)
}

/// 교사 세션 검증 공통 헬퍼 — 다른 라우트 핸들러에서 재사용
pub async fn parse_teacher_session(
    db: &sqlx::SqlitePool,
    headers: &HeaderMap,
) -> Result<(i64, String, String), ApiError> {
    let token = parse_session_token(headers).ok_or(ApiError::Unauthorized)?;

    let row = sqlx::query_as::<_, (i64, String, String)>(
        r#"SELECT t.id, t.name, t.role
           FROM auth_tokens a
           JOIN teachers t ON t.id = a.teacher_id
           WHERE a.token = ? AND a.expires_at > datetime('now')"#,
    )
    .bind(&token)
    .fetch_optional(db)
    .await?;

    row.ok_or(ApiError::Unauthorized)
}

fn parse_student_token(headers: &HeaderMap) -> Option<String> {
    parse_cookie(headers, STUDENT_SESSION_COOKIE)
}

fn set_student_cookie(token: &str) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "{STUDENT_SESSION_COOKIE}={token}; HttpOnly; Path=/; SameSite=Lax; Max-Age={SESSION_MAX_AGE_SECS}"
    ))
    .unwrap()
}

fn clear_student_cookie() -> HeaderValue {
    HeaderValue::from_str(&format!(
        "{STUDENT_SESSION_COOKIE}=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0"
    ))
    .unwrap()
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

#[derive(Deserialize)]
pub struct StudentLoginRequest {
    pub student_number: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct StudentMeResponse {
    pub id: i64,
    pub student_number: String,
    pub name: String,
    pub division_id: i64,
    pub password_reset_required: bool,
}

/// POST /api/auth/login/student
pub async fn student_login(
    State(state): State<AppState>,
    Json(body): Json<StudentLoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let row = sqlx::query_as::<_, (i64, String, String, i64, i64)>(
        "SELECT id, password_hash, name, division_id, password_reset_required
         FROM students WHERE student_number = ?",
    )
    .bind(&body.student_number)
    .fetch_optional(&state.db)
    .await?;

    let (student_id, password_hash, name, division_id, reset_required) =
        row.ok_or(ApiError::Unauthorized)?;

    if password_hash.is_empty() {
        return Err(ApiError::BadRequest("비밀번호가 설정되지 않은 계정입니다".into()));
    }

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

    sqlx::query(
        "INSERT INTO student_sessions (student_id, token, expires_at) VALUES (?, ?, ?)",
    )
    .bind(student_id)
    .bind(&token)
    .bind(&expires_at)
    .execute(&state.db)
    .await?;

    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, set_student_cookie(&token));

    Ok((
        StatusCode::OK,
        headers,
        Json(json!({
            "ok": true,
            "user": {
                "id": student_id,
                "student_number": body.student_number,
                "name": name,
                "division_id": division_id,
                "password_reset_required": reset_required != 0,
            }
        })),
    ))
}

/// POST /api/auth/logout/student
pub async fn student_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(token) = parse_student_token(&headers) {
        sqlx::query("DELETE FROM student_sessions WHERE token = ?")
            .bind(&token)
            .execute(&state.db)
            .await?;
    }

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(header::SET_COOKIE, clear_student_cookie());

    Ok((StatusCode::OK, resp_headers, Json(json!({ "ok": true }))))
}

/// GET /api/auth/student/me
pub async fn student_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<StudentMeResponse>, ApiError> {
    let token = parse_student_token(&headers).ok_or(ApiError::Unauthorized)?;

    let row = sqlx::query_as::<_, (i64, String, String, i64, i64)>(
        r#"SELECT s.id, s.student_number, s.name, s.division_id, s.password_reset_required
           FROM student_sessions ss
           JOIN students s ON s.id = ss.student_id
           WHERE ss.token = ? AND ss.expires_at > datetime('now')"#,
    )
    .bind(&token)
    .fetch_optional(&state.db)
    .await?;

    let (id, student_number, name, division_id, reset_required) =
        row.ok_or(ApiError::Unauthorized)?;

    Ok(Json(StudentMeResponse {
        id,
        student_number,
        name,
        division_id,
        password_reset_required: reset_required != 0,
    }))
}

/// POST /api/auth/student/change-password
#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub async fn student_change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ChangePasswordRequest>,
) -> Result<Json<Value>, ApiError> {

    let token = parse_student_token(&headers).ok_or(ApiError::Unauthorized)?;

    let row = sqlx::query_as::<_, (i64, String)>(
        r#"SELECT s.id, s.password_hash
           FROM student_sessions ss
           JOIN students s ON s.id = ss.student_id
           WHERE ss.token = ? AND ss.expires_at > datetime('now')"#,
    )
    .bind(&token)
    .fetch_optional(&state.db)
    .await?;

    let (student_id, password_hash) = row.ok_or(ApiError::Unauthorized)?;

    if body.new_password.len() < 6 {
        return Err(ApiError::BadRequest("비밀번호는 6자 이상이어야 합니다".into()));
    }

    if !password_hash.is_empty() {
        let parsed = PasswordHash::new(&password_hash)
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        Argon2::default()
            .verify_password(body.current_password.as_bytes(), &parsed)
            .map_err(|_| ApiError::Unauthorized)?;
    }

    let salt = SaltString::generate(&mut OsRng);
    let new_hash = Argon2::default()
        .hash_password(body.new_password.as_bytes(), &salt)
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .to_string();

    sqlx::query(
        "UPDATE students SET password_hash = ?, password_reset_required = 0 WHERE id = ?",
    )
    .bind(&new_hash)
    .bind(student_id)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({ "ok": true })))
}

/// GET /api/auth/school-name
pub async fn school_name(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let name = sqlx::query_scalar::<_, String>("SELECT school_name FROM settings WHERE id = 1")
        .fetch_optional(&state.db)
        .await?
        .unwrap_or_default();

    Ok(Json(json!({ "school_name": name })))
}
