use axum::{extract::State, http::{header, HeaderMap, HeaderValue}, response::IntoResponse, Json};
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier, PasswordHasher, SaltString},
    Argon2,
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{error::ApiError, server::state::AppState};

// ── Session parsing ────────────────────────────────────────────────────────────

pub struct SessionInfo {
    pub session_id: i64,
    pub teacher_id: Option<i64>,
    pub student_id: Option<i64>,
}

fn get_cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookie_str = headers.get("cookie")?.to_str().ok()?;
    cookie_str.split(';').find_map(|part| {
        let part = part.trim();
        let prefix = format!("{name}=");
        part.strip_prefix(&prefix).map(|v| v.to_string())
    })
}

pub async fn parse_session(
    headers: &HeaderMap,
    db: &sqlx::SqlitePool,
) -> Result<SessionInfo, ApiError> {
    let token = get_cookie_value(headers, "cc_session").ok_or(ApiError::Unauthorized)?;

    let row = sqlx::query(
        "SELECT id, teacher_id, student_id FROM auth_sessions \
         WHERE token = ? AND expires_at > datetime('now')",
    )
    .bind(&token)
    .fetch_optional(db)
    .await?;

    match row {
        None => Err(ApiError::Unauthorized),
        Some(r) => {
            use sqlx::Row as _;
            Ok(SessionInfo {
                session_id: r.get("id"),
                teacher_id: r.get("teacher_id"),
                student_id: r.get("student_id"),
            })
        }
    }
}

fn set_session_cookie(token: &str) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "cc_session={token}; HttpOnly; SameSite=Lax; Max-Age=43200; Path=/"
    ))
    .expect("valid header value")
}

fn clear_session_cookie() -> HeaderValue {
    HeaderValue::from_str("cc_session=; HttpOnly; SameSite=Lax; Max-Age=0; Path=/")
        .expect("valid header value")
}

async fn get_default_locale(db: &sqlx::SqlitePool) -> String {
    sqlx::query_scalar::<_, String>("SELECT value FROM app_configs WHERE key = 'locale'")
        .fetch_optional(db)
        .await
        .unwrap_or(None)
        .unwrap_or_else(|| "ko".to_string())
}

// ── Teacher login ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct TeacherLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct TeacherUser {
    pub id: i64,
    pub username: String,
    pub name: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct TeacherLoginResponse {
    pub user: TeacherUser,
    pub locale: String,
}

/// POST /api/auth/login/teacher
pub async fn login_teacher(
    State(state): State<AppState>,
    Json(body): Json<TeacherLoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // 기존 세션이 있으면 교체 (재로그인 허용)
    struct TeacherRow {
        id: i64,
        username: String,
        name: String,
        role: String,
        password_hash: String,
    }

    let teacher = sqlx::query(
        "SELECT id, username, name, role, password_hash FROM teachers WHERE username = ?",
    )
    .bind(&body.username)
    .fetch_optional(&state.db)
    .await?;

    let teacher = match teacher {
        None => return Err(ApiError::BadRequest("ERR_INVALID_CREDENTIALS".into())),
        Some(r) => {
            use sqlx::Row as _;
            TeacherRow {
                id: r.get("id"),
                username: r.get("username"),
                name: r.get("name"),
                role: r.get("role"),
                password_hash: r.get("password_hash"),
            }
        }
    };

    let parsed_hash = PasswordHash::new(&teacher.password_hash)
        .map_err(|_| ApiError::BadRequest("ERR_INVALID_CREDENTIALS".into()))?;
    Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .map_err(|_| ApiError::BadRequest("ERR_INVALID_CREDENTIALS".into()))?;

    let token = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO auth_sessions (token, teacher_id, expires_at) \
         VALUES (?, ?, datetime('now', '+12 hours'))",
    )
    .bind(&token)
    .bind(teacher.id)
    .execute(&state.db)
    .await?;

    // 교사 개인 locale 조회 (없으면 앱 기본값)
    let locale = sqlx::query_scalar::<_, String>(
        "SELECT value FROM teacher_settings WHERE teacher_id = ? AND key = 'locale'",
    )
    .bind(teacher.id)
    .fetch_optional(&state.db)
    .await?
    .unwrap_or_default();

    let locale = if locale.is_empty() {
        get_default_locale(&state.db).await
    } else {
        locale
    };

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(header::SET_COOKIE, set_session_cookie(&token));

    Ok((
        resp_headers,
        Json(TeacherLoginResponse {
            user: TeacherUser {
                id: teacher.id,
                username: teacher.username,
                name: teacher.name,
                role: teacher.role,
            },
            locale,
        }),
    ))
}

// ── Teacher logout ─────────────────────────────────────────────────────────────

/// POST /api/auth/logout
pub async fn logout_teacher(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(token) = get_cookie_value(&headers, "cc_session") {
        sqlx::query(
            "DELETE FROM auth_sessions WHERE token = ? AND teacher_id IS NOT NULL",
        )
        .bind(&token)
        .execute(&state.db)
        .await?;
    }

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(header::SET_COOKIE, clear_session_cookie());
    Ok((resp_headers, Json(json!({ "ok": true }))))
}

// ── Teacher me ────────────────────────────────────────────────────────────────

/// GET /api/auth/me
pub async fn me_teacher(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<TeacherUser>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;

    let row = sqlx::query(
        "SELECT id, username, name, role FROM teachers WHERE id = ?",
    )
    .bind(teacher_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    use sqlx::Row as _;
    Ok(Json(TeacherUser {
        id: row.get("id"),
        username: row.get("username"),
        name: row.get("name"),
        role: row.get("role"),
    }))
}

// ── School name ───────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct SchoolNameResponse {
    pub school_name: String,
}

/// GET /api/auth/school-name
pub async fn school_name(
    State(state): State<AppState>,
) -> Result<Json<SchoolNameResponse>, ApiError> {
    let name = sqlx::query_scalar::<_, String>(
        "SELECT value FROM app_configs WHERE key = 'school_name'",
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(SchoolNameResponse { school_name: name }))
}

// ── Student login ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct StudentLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct StudentUser {
    pub id: i64,
    pub username: String,
    pub name: String,
    pub grade: i64,
    pub class_no: i64,
    pub number: i64,
    pub password_reset_required: bool,
}

#[derive(Serialize)]
pub struct StudentLoginResponse {
    pub user: StudentUser,
    pub locale: String,
}

/// POST /api/auth/login/student
pub async fn login_student(
    State(state): State<AppState>,
    Json(body): Json<StudentLoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    struct StudentRow {
        id: i64,
        username: String,
        name: String,
        grade: i64,
        class_no: i64,
        number: i64,
        password_hash: String,
        password_reset_required: i64,
    }

    let student = sqlx::query(
        "SELECT id, username, name, grade, class_no, number, password_hash, \
         password_reset_required FROM students WHERE username = ?",
    )
    .bind(&body.username)
    .fetch_optional(&state.db)
    .await?;

    let student = match student {
        None => return Err(ApiError::BadRequest("ERR_INVALID_CREDENTIALS".into())),
        Some(r) => {
            use sqlx::Row as _;
            StudentRow {
                id: r.get("id"),
                username: r.get("username"),
                name: r.get("name"),
                grade: r.get("grade"),
                class_no: r.get("class_no"),
                number: r.get("number"),
                password_hash: r.get("password_hash"),
                password_reset_required: r.get("password_reset_required"),
            }
        }
    };

    // 비밀번호 검증 (hash가 비어있으면 로그인 불가)
    if student.password_hash.is_empty() {
        return Err(ApiError::BadRequest("ERR_INVALID_CREDENTIALS".into()));
    }

    let parsed_hash = PasswordHash::new(&student.password_hash)
        .map_err(|_| ApiError::BadRequest("ERR_INVALID_CREDENTIALS".into()))?;
    Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .map_err(|_| ApiError::BadRequest("ERR_INVALID_CREDENTIALS".into()))?;

    let token = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO auth_sessions (token, student_id, expires_at) \
         VALUES (?, ?, datetime('now', '+12 hours'))",
    )
    .bind(&token)
    .bind(student.id)
    .execute(&state.db)
    .await?;

    // 학생 개인 locale 조회 (없으면 앱 기본값)
    let locale = sqlx::query_scalar::<_, String>(
        "SELECT value FROM student_settings WHERE student_id = ? AND key = 'locale'",
    )
    .bind(student.id)
    .fetch_optional(&state.db)
    .await?
    .unwrap_or_else(|| "".to_string());

    let locale = if locale.is_empty() {
        get_default_locale(&state.db).await
    } else {
        locale
    };

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(header::SET_COOKIE, set_session_cookie(&token));

    Ok((
        resp_headers,
        Json(StudentLoginResponse {
            user: StudentUser {
                id: student.id,
                username: student.username,
                name: student.name,
                grade: student.grade,
                class_no: student.class_no,
                number: student.number,
                password_reset_required: student.password_reset_required == 1,
            },
            locale,
        }),
    ))
}

// ── Student logout ────────────────────────────────────────────────────────────

/// POST /api/auth/logout/student
pub async fn logout_student(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(token) = get_cookie_value(&headers, "cc_session") {
        sqlx::query(
            "DELETE FROM auth_sessions WHERE token = ? AND student_id IS NOT NULL",
        )
        .bind(&token)
        .execute(&state.db)
        .await?;
    }

    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(header::SET_COOKIE, clear_session_cookie());
    Ok((resp_headers, Json(json!({ "ok": true }))))
}

// ── Student me ────────────────────────────────────────────────────────────────

/// GET /api/auth/student/me
pub async fn me_student(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<StudentUser>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let student_id = session.student_id.ok_or(ApiError::Forbidden)?;

    let row = sqlx::query(
        "SELECT id, username, name, grade, class_no, number, password_reset_required \
         FROM students WHERE id = ?",
    )
    .bind(student_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    use sqlx::Row as _;
    Ok(Json(StudentUser {
        id: row.get("id"),
        username: row.get("username"),
        name: row.get("name"),
        grade: row.get("grade"),
        class_no: row.get("class_no"),
        number: row.get("number"),
        password_reset_required: {
            let v: i64 = row.get("password_reset_required");
            v == 1
        },
    }))
}

// ── Teacher update name ───────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct UpdateTeacherNameRequest {
    pub name: String,
}

/// PUT /api/auth/me
pub async fn update_teacher_name(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UpdateTeacherNameRequest>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;

    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::BadRequest("ERR_NAME_REQUIRED".into()));
    }

    sqlx::query("UPDATE teachers SET name = ? WHERE id = ?")
        .bind(&name)
        .bind(teacher_id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "ok": true })))
}

// ── Teacher change password ───────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ChangePasswordTeacherRequest {
    pub current_password: String,
    pub new_password: String,
}

/// PUT /api/auth/me/password
pub async fn change_password_teacher(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ChangePasswordTeacherRequest>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;

    if body.new_password.len() < 8 {
        return Err(ApiError::BadRequest("ERR_PASSWORD_TOO_SHORT".into()));
    }

    let row = sqlx::query("SELECT password_hash FROM teachers WHERE id = ?")
        .bind(teacher_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(ApiError::NotFound)?;

    use sqlx::Row as _;
    let current_hash: String = row.get("password_hash");

    let parsed_hash = PasswordHash::new(&current_hash)
        .map_err(|_| ApiError::BadRequest("ERR_INVALID_CREDENTIALS".into()))?;
    Argon2::default()
        .verify_password(body.current_password.as_bytes(), &parsed_hash)
        .map_err(|_| ApiError::BadRequest("ERR_INVALID_CREDENTIALS".into()))?;

    let salt = SaltString::generate(&mut OsRng);
    let new_hash = Argon2::default()
        .hash_password(body.new_password.as_bytes(), &salt)
        .map_err(|e| ApiError::Internal(format!("argon2: {e}")))?
        .to_string();

    sqlx::query("UPDATE teachers SET password_hash = ? WHERE id = ?")
        .bind(&new_hash)
        .bind(teacher_id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "ok": true })))
}

// ── Student change password ───────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: Option<String>,
    pub new_password: String,
}

/// POST /api/auth/student/change-password
pub async fn change_password_student(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ChangePasswordRequest>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let student_id = session.student_id.ok_or(ApiError::Forbidden)?;

    if body.new_password.len() < 8 {
        return Err(ApiError::BadRequest("ERR_PASSWORD_TOO_SHORT".into()));
    }

    struct PwRow {
        password_hash: String,
        password_reset_required: i64,
    }

    let row = sqlx::query(
        "SELECT password_hash, password_reset_required FROM students WHERE id = ?",
    )
    .bind(student_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    use sqlx::Row as _;
    let pw_row = PwRow {
        password_hash: row.get("password_hash"),
        password_reset_required: row.get("password_reset_required"),
    };

    // 기존 비밀번호가 있는 경우 반드시 검증
    let skip_current_check = pw_row.password_hash.is_empty() && pw_row.password_reset_required == 1;

    if !skip_current_check {
        let current = body
            .current_password
            .as_deref()
            .ok_or_else(|| ApiError::BadRequest("ERR_CURRENT_PASSWORD_REQUIRED".into()))?;

        let parsed_hash = PasswordHash::new(&pw_row.password_hash)
            .map_err(|_| ApiError::BadRequest("ERR_INVALID_CREDENTIALS".into()))?;
        Argon2::default()
            .verify_password(current.as_bytes(), &parsed_hash)
            .map_err(|_| ApiError::BadRequest("ERR_INVALID_CREDENTIALS".into()))?;
    }

    let salt = SaltString::generate(&mut OsRng);
    let new_hash = Argon2::default()
        .hash_password(body.new_password.as_bytes(), &salt)
        .map_err(|e| ApiError::Internal(format!("argon2: {e}")))?
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
