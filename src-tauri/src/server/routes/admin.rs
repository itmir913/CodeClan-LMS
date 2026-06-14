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

    let result = sqlx::query("INSERT INTO subjects (name) VALUES (?)")
        .bind(body.name.trim())
        .execute(&state.db)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.message().contains("UNIQUE constraint failed") {
                    return ApiError::BadRequest("ERR_SUBJECT_NAME_TAKEN".into());
                }
            }
            ApiError::Database(e)
        })?;

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

    // 이 과목을 사용하는 수업이 있으면 삭제 불가
    let in_use: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM classes WHERE subject_id = ? LIMIT 1",
    )
    .bind(subject_id)
    .fetch_optional(&state.db)
    .await?;
    if in_use.is_some() {
        return Err(ApiError::BadRequest("ERR_SUBJECT_IN_USE".into()));
    }

    let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM subjects WHERE id = ?")
        .bind(subject_id)
        .fetch_optional(&state.db)
        .await?;
    if exists.is_none() {
        return Err(ApiError::NotFound);
    }

    sqlx::query("DELETE FROM subjects WHERE id = ?")
        .bind(subject_id)
        .execute(&state.db)
        .await?;

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
        "SELECT id, username, name, role, created_at FROM teachers ORDER BY created_at ASC",
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

    let result = sqlx::query(
        "INSERT INTO teachers (username, name, password_hash, role) VALUES (?, ?, ?, ?)",
    )
    .bind(body.username.trim())
    .bind(body.name.trim())
    .bind(&hash)
    .bind(role)
    .execute(&state.db)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.message().contains("UNIQUE constraint failed") {
                return ApiError::BadRequest("ERR_USERNAME_TAKEN".into());
            }
        }
        ApiError::Database(e)
    })?;

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

    if let Some(ref password) = body.password {
        if !password.is_empty() {
            let salt = SaltString::generate(&mut OsRng);
            let hash = Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map_err(|e| ApiError::Internal(format!("argon2: {e}")))?
                .to_string();
            sqlx::query("UPDATE teachers SET password_hash = ? WHERE id = ?")
                .bind(&hash)
                .bind(target_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;
    Ok(Json(json!({ "ok": true })))
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
