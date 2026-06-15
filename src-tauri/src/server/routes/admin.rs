use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{error::ApiError, server::{routes::auth::parse_session, state::AppState}};

async fn require_admin(teacher_id: i64, db: &sqlx::SqlitePool) -> Result<(), ApiError> {
    let role: String = sqlx::query_scalar("SELECT role FROM teachers WHERE id = ?")
        .bind(teacher_id)
        .fetch_optional(db)
        .await?
        .ok_or(ApiError::NotFound)?;
    if role != "admin" {
        return Err(ApiError::Forbidden);
    }
    Ok(())
}

// ── App settings ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct AppSettingsResponse {
    pub school_name: String,
    pub locale: String,
}

/// GET /api/admin/app-settings
pub async fn get_app_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AppSettingsResponse>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;
    require_admin(teacher_id, &state.db).await?;

    let school_name = sqlx::query_scalar::<_, String>(
        "SELECT value FROM app_configs WHERE key = 'school_name'",
    )
    .fetch_optional(&state.db)
    .await?
    .unwrap_or_default();

    let locale = sqlx::query_scalar::<_, String>(
        "SELECT value FROM app_configs WHERE key = 'locale'",
    )
    .fetch_optional(&state.db)
    .await?
    .unwrap_or_else(|| "ko".to_string());

    Ok(Json(AppSettingsResponse { school_name, locale }))
}

#[derive(Deserialize)]
pub struct UpdateAppSettingsRequest {
    pub school_name: String,
    pub locale: String,
}

/// PUT /api/admin/app-settings
pub async fn update_app_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UpdateAppSettingsRequest>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;
    require_admin(teacher_id, &state.db).await?;

    let school_name = body.school_name.trim().to_string();
    if school_name.is_empty() {
        return Err(ApiError::BadRequest("ERR_SCHOOL_NAME_REQUIRED".into()));
    }

    let raw_locale = body.locale.trim().to_string();
    let locale = if raw_locale.is_empty()
        || raw_locale.len() > 10
        || !raw_locale.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(ApiError::BadRequest("ERR_INVALID_LOCALE".into()));
    } else {
        raw_locale
    };

    let mut tx = state.db.begin().await?;
    sqlx::query(
        "INSERT INTO app_configs (key, value) VALUES ('school_name', ?) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(&school_name)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO app_configs (key, value) VALUES ('locale', ?) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(&locale)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(Json(json!({ "ok": true })))
}

// ── Subjects ──────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct SubjectRow {
    pub id: i64,
    pub name: String,
}

#[derive(Deserialize)]
pub struct SubjectBody {
    pub name: String,
}

/// GET /api/subjects  (교사·admin 공용 — 수업 생성 폼에서 사용)
pub async fn list_subjects(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<SubjectRow>>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    session.teacher_id.ok_or(ApiError::Forbidden)?;

    let rows = sqlx::query("SELECT id, name FROM subjects ORDER BY name ASC")
        .fetch_all(&state.db)
        .await?;

    use sqlx::Row as _;
    Ok(Json(
        rows.iter()
            .map(|r| SubjectRow { id: r.get("id"), name: r.get("name") })
            .collect(),
    ))
}

/// POST /api/admin/subjects
pub async fn create_subject(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SubjectBody>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;
    require_admin(teacher_id, &state.db).await?;

    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("ERR_SUBJECT_NAME_REQUIRED".into()));
    }

    let mut tx = state.db.begin().await?;
    let result = sqlx::query("INSERT INTO subjects (name) VALUES (?)")
        .bind(body.name.trim())
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.message().contains("UNIQUE constraint failed") {
                    return ApiError::BadRequest("ERR_SUBJECT_NAME_TAKEN".into());
                }
            }
            ApiError::Database(e)
        })?;
    tx.commit().await?;

    Ok(Json(json!({ "id": result.last_insert_rowid() })))
}

/// DELETE /api/admin/subjects/:id
pub async fn delete_subject(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(subject_id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;
    require_admin(teacher_id, &state.db).await?;

    let mut tx = state.db.begin().await?;

    let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM subjects WHERE id = ?")
        .bind(subject_id)
        .fetch_optional(&mut *tx)
        .await?;
    if exists.is_none() {
        return Err(ApiError::NotFound);
    }

    // 이 과목을 사용하는 수업이 있으면 삭제 불가
    let in_use: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM classes WHERE subject_id = ? LIMIT 1",
    )
    .bind(subject_id)
    .fetch_optional(&mut *tx)
    .await?;
    if in_use.is_some() {
        return Err(ApiError::BadRequest("ERR_SUBJECT_IN_USE".into()));
    }

    sqlx::query("DELETE FROM subjects WHERE id = ?")
        .bind(subject_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(Json(json!({ "ok": true })))
}

// ── Teachers ──────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct AdminTeacher {
    pub id: i64,
    pub username: String,
    pub name: String,
    pub role: String,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct CreateTeacherRequest {
    pub username: String,
    pub name: String,
    pub password: String,
    pub role: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateTeacherRequest {
    pub name: Option<String>,
    pub role: Option<String>,
    pub password: Option<String>,
}

/// GET /api/admin/teachers
pub async fn list_teachers(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<AdminTeacher>>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;
    require_admin(teacher_id, &state.db).await?;

    let rows = sqlx::query(
        "SELECT id, username, name, role, created_at FROM teachers ORDER BY CASE role WHEN 'admin' THEN 0 ELSE 1 END ASC, name ASC",
    )
    .fetch_all(&state.db)
    .await?;

    use sqlx::Row as _;
    Ok(Json(
        rows.iter()
            .map(|r| AdminTeacher {
                id: r.get("id"),
                username: r.get("username"),
                name: r.get("name"),
                role: r.get("role"),
                created_at: r.get("created_at"),
            })
            .collect(),
    ))
}

/// POST /api/admin/teachers
pub async fn create_teacher(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateTeacherRequest>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;
    require_admin(teacher_id, &state.db).await?;

    if body.username.trim().is_empty() {
        return Err(ApiError::BadRequest("ERR_USERNAME_REQUIRED".into()));
    }
    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("ERR_ADMIN_NAME_REQUIRED".into()));
    }
    if body.password.len() < 8 {
        return Err(ApiError::BadRequest("ERR_PASSWORD_TOO_SHORT".into()));
    }

    let role = if body.role.as_deref() == Some("admin") { "admin" } else { "teacher" };

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| ApiError::Internal(format!("argon2: {e}")))?
        .to_string();

    let mut tx = state.db.begin().await?;
    let result = sqlx::query(
        "INSERT INTO teachers (username, name, password_hash, role) VALUES (?, ?, ?, ?)",
    )
    .bind(body.username.trim())
    .bind(body.name.trim())
    .bind(&hash)
    .bind(role)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.message().contains("UNIQUE constraint failed") {
                return ApiError::BadRequest("ERR_USERNAME_TAKEN".into());
            }
        }
        ApiError::Database(e)
    })?;
    tx.commit().await?;

    Ok(Json(json!({ "id": result.last_insert_rowid() })))
}

/// PUT /api/admin/teachers/:id
pub async fn update_teacher(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(target_id): Path<i64>,
    Json(body): Json<UpdateTeacherRequest>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;
    require_admin(teacher_id, &state.db).await?;

    if let Some(ref name) = body.name {
        if name.trim().is_empty() {
            return Err(ApiError::BadRequest("ERR_ADMIN_NAME_REQUIRED".into()));
        }
    }
    if let Some(ref password) = body.password {
        if !password.is_empty() && password.len() < 8 {
            return Err(ApiError::BadRequest("ERR_PASSWORD_TOO_SHORT".into()));
        }
    }

    // 해싱은 CPU 집약 작업이므로 트랜잭션 밖에서 미리 계산
    let new_password_hash: Option<String> = match &body.password {
        Some(pw) if !pw.is_empty() => {
            let salt = SaltString::generate(&mut OsRng);
            Some(
                Argon2::default()
                    .hash_password(pw.as_bytes(), &salt)
                    .map_err(|e| ApiError::Internal(format!("argon2: {e}")))?
                    .to_string(),
            )
        }
        _ => None,
    };

    let mut tx = state.db.begin().await?;

    let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM teachers WHERE id = ?")
        .bind(target_id)
        .fetch_optional(&mut *tx)
        .await?;
    if exists.is_none() {
        return Err(ApiError::NotFound);
    }

    // role을 teacher로 내리려는 경우 마지막 관리자 보호
    if let Some(ref role) = body.role {
        if role != "admin" {
            let admin_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM teachers WHERE role = 'admin'")
                    .fetch_one(&mut *tx)
                    .await?;
            let target_is_admin: Option<i64> = sqlx::query_scalar(
                "SELECT id FROM teachers WHERE id = ? AND role = 'admin'",
            )
            .bind(target_id)
            .fetch_optional(&mut *tx)
            .await?;
            if target_is_admin.is_some() && admin_count <= 1 {
                return Err(ApiError::BadRequest("ERR_LAST_ADMIN".into()));
            }
        }
    }

    if let Some(ref name) = body.name {
        sqlx::query("UPDATE teachers SET name = ? WHERE id = ?")
            .bind(name.trim())
            .bind(target_id)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(ref role) = body.role {
        let role_val = if role == "admin" { "admin" } else { "teacher" };
        sqlx::query("UPDATE teachers SET role = ? WHERE id = ?")
            .bind(role_val)
            .bind(target_id)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(ref hash) = new_password_hash {
        sqlx::query("UPDATE teachers SET password_hash = ? WHERE id = ?")
            .bind(hash)
            .bind(target_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(Json(json!({ "ok": true })))
}

// ── Bulk Import ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ImportTeacherRow {
    pub username: String,
    pub name: String,
    pub password: String,
    pub role: Option<String>,
}

/// POST /api/admin/teachers/import
pub async fn import_teachers(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Vec<ImportTeacherRow>>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;
    require_admin(teacher_id, &state.db).await?;

    if body.is_empty() {
        return Err(ApiError::BadRequest("ERR_IMPORT_EMPTY".into()));
    }

    // 전체 행 유효성 검사 (tx 밖)
    for item in &body {
        if item.username.trim().is_empty() || item.name.trim().is_empty() {
            return Err(ApiError::BadRequest("ERR_IMPORT_ROW_INVALID".into()));
        }
        if item.password.len() < 8 {
            return Err(ApiError::BadRequest("ERR_IMPORT_PASSWORD_TOO_SHORT".into()));
        }
    }

    // username 중복 확인 (tx 밖, &state.db로 별도 커넥션)
    let mut skipped_items: Vec<String> = Vec::new();
    let mut new_items: Vec<&ImportTeacherRow> = Vec::new();
    for item in &body {
        let exists: Option<i64> =
            sqlx::query_scalar("SELECT id FROM teachers WHERE username = ?")
                .bind(item.username.trim())
                .fetch_optional(&state.db)
                .await?;
        if exists.is_some() {
            skipped_items.push(item.username.trim().to_string());
        } else {
            new_items.push(item);
        }
    }

    // 신규 교사만 해싱 (tx 밖)
    struct PreparedTeacher {
        username: String,
        name: String,
        hash: String,
        role: &'static str,
    }
    let mut prepared: Vec<PreparedTeacher> = Vec::with_capacity(new_items.len());
    for item in &new_items {
        let role = if item.role.as_deref() == Some("admin") { "admin" } else { "teacher" };
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(item.password.as_bytes(), &salt)
            .map_err(|e| ApiError::Internal(format!("argon2: {e}")))?
            .to_string();
        prepared.push(PreparedTeacher {
            username: item.username.trim().to_string(),
            name: item.name.trim().to_string(),
            hash,
            role,
        });
    }

    if !prepared.is_empty() {
        let mut tx = state.db.begin().await?;
        for p in &prepared {
            sqlx::query(
                "INSERT INTO teachers (username, name, password_hash, role) VALUES (?, ?, ?, ?)",
            )
            .bind(&p.username)
            .bind(&p.name)
            .bind(&p.hash)
            .bind(p.role)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
    }

    Ok(Json(json!({
        "imported": prepared.len(),
        "skipped": skipped_items.len(),
        "skipped_items": skipped_items
    })))
}

/// POST /api/admin/subjects/import
pub async fn import_subjects(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Vec<SubjectBody>>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;
    require_admin(teacher_id, &state.db).await?;

    if body.is_empty() {
        return Err(ApiError::BadRequest("ERR_IMPORT_EMPTY".into()));
    }

    for item in &body {
        if item.name.trim().is_empty() {
            return Err(ApiError::BadRequest("ERR_IMPORT_ROW_INVALID".into()));
        }
    }

    let mut tx = state.db.begin().await?;
    let mut imported: usize = 0;
    let mut skipped_items: Vec<String> = Vec::new();

    for item in &body {
        let result = sqlx::query("INSERT OR IGNORE INTO subjects (name) VALUES (?)")
            .bind(item.name.trim())
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 0 {
            skipped_items.push(item.name.trim().to_string());
        } else {
            imported += 1;
        }
    }

    tx.commit().await?;
    Ok(Json(json!({
        "imported": imported,
        "skipped": skipped_items.len(),
        "skipped_items": skipped_items
    })))
}

/// DELETE /api/admin/teachers/:id
pub async fn delete_teacher(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(target_id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;
    require_admin(teacher_id, &state.db).await?;

    if target_id == teacher_id {
        return Err(ApiError::BadRequest("ERR_CANNOT_DELETE_SELF".into()));
    }

    let mut tx = state.db.begin().await?;

    let target_role: Option<String> =
        sqlx::query_scalar("SELECT role FROM teachers WHERE id = ?")
            .bind(target_id)
            .fetch_optional(&mut *tx)
            .await?;
    let target_role = target_role.ok_or(ApiError::NotFound)?;

    // 마지막 관리자 삭제 방지
    if target_role == "admin" {
        let admin_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM teachers WHERE role = 'admin'")
                .fetch_one(&mut *tx)
                .await?;
        if admin_count <= 1 {
            return Err(ApiError::BadRequest("ERR_LAST_ADMIN".into()));
        }
    }

    sqlx::query("DELETE FROM teachers WHERE id = ?")
        .bind(target_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(Json(json!({ "ok": true })))
}
