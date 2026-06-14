use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{error::ApiError, server::{routes::auth::parse_session, state::AppState}};

#[derive(Serialize)]
pub struct ClassResponse {
    pub id: i64,
    pub name: String,
    pub subject_id: i64,
    pub subject_name: String,
    pub teacher_id: i64,
    pub student_count: i64,
    pub has_active_session: bool,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct ClassBody {
    pub name: String,
    pub subject_id: i64,
}

async fn check_class_access(
    teacher_id: i64,
    class_id: i64,
    role: &str,
    db: &sqlx::SqlitePool,
) -> Result<(), ApiError> {
    if role == "admin" {
        let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM classes WHERE id = ?")
            .bind(class_id)
            .fetch_optional(db)
            .await?;
        exists.ok_or(ApiError::NotFound)?;
    } else {
        let owned: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM classes WHERE id = ? AND teacher_id = ?",
        )
        .bind(class_id)
        .bind(teacher_id)
        .fetch_optional(db)
        .await?;
        owned.ok_or(ApiError::Forbidden)?;
    }
    Ok(())
}

/// GET /api/classes
pub async fn list_classes(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ClassResponse>>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;

    let role: String = sqlx::query_scalar("SELECT role FROM teachers WHERE id = ?")
        .bind(teacher_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(ApiError::NotFound)?;

    let rows = if role == "admin" {
        sqlx::query(
            "SELECT c.id, c.name, c.subject_id, c.teacher_id, c.created_at, \
             COALESCE(s.name, '') as subject_name, \
             COUNT(cs.student_id) as student_count \
             FROM classes c \
             LEFT JOIN subjects s ON s.id = c.subject_id \
             LEFT JOIN class_students cs ON cs.class_id = c.id \
             GROUP BY c.id \
             ORDER BY c.created_at DESC",
        )
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query(
            "SELECT c.id, c.name, c.subject_id, c.teacher_id, c.created_at, \
             COALESCE(s.name, '') as subject_name, \
             COUNT(cs.student_id) as student_count \
             FROM classes c \
             LEFT JOIN subjects s ON s.id = c.subject_id \
             LEFT JOIN class_students cs ON cs.class_id = c.id \
             WHERE c.teacher_id = ? \
             GROUP BY c.id \
             ORDER BY c.created_at DESC",
        )
        .bind(teacher_id)
        .fetch_all(&state.db)
        .await?
    };

    use sqlx::Row as _;
    let classes = rows
        .iter()
        .map(|r| ClassResponse {
            id: r.get("id"),
            name: r.get("name"),
            subject_id: r.get("subject_id"),
            subject_name: r.get("subject_name"),
            teacher_id: r.get("teacher_id"),
            student_count: r.get("student_count"),
            has_active_session: false, // 단계 8에서 구현
            created_at: r.get("created_at"),
        })
        .collect();

    Ok(Json(classes))
}

/// POST /api/classes
pub async fn create_class(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ClassBody>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;

    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("ERR_CLASS_NAME_REQUIRED".into()));
    }

    let mut tx = state.db.begin().await?;

    let subject_exists: Option<i64> = sqlx::query_scalar("SELECT id FROM subjects WHERE id = ?")
        .bind(body.subject_id)
        .fetch_optional(&mut *tx)
        .await?;
    if subject_exists.is_none() {
        return Err(ApiError::BadRequest("ERR_SUBJECT_NOT_FOUND".into()));
    }

    let result = sqlx::query(
        "INSERT INTO classes (teacher_id, subject_id, name) VALUES (?, ?, ?)",
    )
    .bind(teacher_id)
    .bind(body.subject_id)
    .bind(body.name.trim())
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(json!({ "id": result.last_insert_rowid() })))
}

/// PUT /api/classes/:id
pub async fn update_class(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(class_id): Path<i64>,
    Json(body): Json<ClassBody>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;

    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("ERR_CLASS_NAME_REQUIRED".into()));
    }

    let role: String = sqlx::query_scalar("SELECT role FROM teachers WHERE id = ?")
        .bind(teacher_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(ApiError::NotFound)?;

    // 접근 권한 확인 (pool 기반 read, tx 밖에서 수행)
    check_class_access(teacher_id, class_id, &role, &state.db).await?;

    let mut tx = state.db.begin().await?;

    let subject_exists: Option<i64> = sqlx::query_scalar("SELECT id FROM subjects WHERE id = ?")
        .bind(body.subject_id)
        .fetch_optional(&mut *tx)
        .await?;
    if subject_exists.is_none() {
        return Err(ApiError::BadRequest("ERR_SUBJECT_NOT_FOUND".into()));
    }

    sqlx::query("UPDATE classes SET name = ?, subject_id = ? WHERE id = ?")
        .bind(body.name.trim())
        .bind(body.subject_id)
        .bind(class_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(Json(json!({ "ok": true })))
}

/// DELETE /api/classes/:id
pub async fn delete_class(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(class_id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;

    let role: String = sqlx::query_scalar("SELECT role FROM teachers WHERE id = ?")
        .bind(teacher_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(ApiError::NotFound)?;

    check_class_access(teacher_id, class_id, &role, &state.db).await?;

    let mut tx = state.db.begin().await?;
    sqlx::query("DELETE FROM classes WHERE id = ?")
        .bind(class_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok(Json(json!({ "ok": true })))
}
