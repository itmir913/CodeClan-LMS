use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{error::ApiError, server::{routes::auth::parse_session, state::AppState}};

#[derive(Serialize)]
pub struct StudentItem {
    pub id: i64,
    pub username: String,
    pub name: String,
    pub grade: i64,
    pub class_no: i64,
    pub number: i64,
    pub password_reset_required: bool,
}

#[derive(Deserialize)]
pub struct AddStudentBody {
    pub name: String,
    pub grade: i64,
    pub class_no: i64,
    pub number: i64,
}

async fn require_class_access(
    teacher_id: i64,
    class_id: i64,
    db: &sqlx::SqlitePool,
) -> Result<(), ApiError> {
    let role: String = sqlx::query_scalar("SELECT role FROM teachers WHERE id = ?")
        .bind(teacher_id)
        .fetch_optional(db)
        .await?
        .ok_or(ApiError::Forbidden)?;

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

async fn require_student_access(
    teacher_id: i64,
    student_id: i64,
    db: &sqlx::SqlitePool,
) -> Result<(), ApiError> {
    let role: String = sqlx::query_scalar("SELECT role FROM teachers WHERE id = ?")
        .bind(teacher_id)
        .fetch_optional(db)
        .await?
        .ok_or(ApiError::Forbidden)?;

    if role != "admin" {
        let has_access: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM class_students cs \
             INNER JOIN classes c ON c.id = cs.class_id \
             WHERE cs.student_id = ? AND c.teacher_id = ? LIMIT 1",
        )
        .bind(student_id)
        .bind(teacher_id)
        .fetch_optional(db)
        .await?;
        if has_access.is_none() {
            return Err(ApiError::Forbidden);
        }
    }
    Ok(())
}

/// GET /api/student/classes
pub async fn list_my_classes(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::server::routes::classes::ClassResponse>>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let student_id = session.student_id.ok_or(ApiError::Forbidden)?;

    let rows = sqlx::query(
        "SELECT c.id, c.name, c.subject_id, c.teacher_id, c.created_at, \
         COALESCE(s.name, '') as subject_name, \
         COALESCE(t.name, '') as teacher_name, \
         (SELECT COUNT(*) FROM class_students cs2 WHERE cs2.class_id = c.id) as student_count \
         FROM class_students cs \
         INNER JOIN classes c ON c.id = cs.class_id \
         LEFT JOIN subjects s ON s.id = c.subject_id \
         LEFT JOIN teachers t ON t.id = c.teacher_id \
         WHERE cs.student_id = ? \
         ORDER BY s.name ASC, c.name ASC",
    )
    .bind(student_id)
    .fetch_all(&state.db)
    .await?;

    use sqlx::Row as _;
    let classes = rows
        .iter()
        .map(|r| crate::server::routes::classes::ClassResponse {
            id: r.get("id"),
            name: r.get("name"),
            subject_id: r.get("subject_id"),
            subject_name: r.get("subject_name"),
            teacher_id: r.get("teacher_id"),
            teacher_name: r.get("teacher_name"),
            student_count: r.get("student_count"),
            has_active_session: false,
            created_at: r.get("created_at"),
        })
        .collect();

    Ok(Json(classes))
}

/// GET /api/classes/:id/students
pub async fn list_students(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(class_id): Path<i64>,
) -> Result<Json<Vec<StudentItem>>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;
    require_class_access(teacher_id, class_id, &state.db).await?;

    let rows = sqlx::query(
        "SELECT s.id, s.username, s.name, s.grade, s.class_no, s.number, s.password_reset_required \
         FROM students s \
         INNER JOIN class_students cs ON cs.student_id = s.id \
         WHERE cs.class_id = ? \
         ORDER BY s.grade, s.class_no, s.number",
    )
    .bind(class_id)
    .fetch_all(&state.db)
    .await?;

    use sqlx::Row as _;
    Ok(Json(
        rows.iter()
            .map(|r| StudentItem {
                id: r.get("id"),
                username: r.get("username"),
                name: r.get("name"),
                grade: r.get("grade"),
                class_no: r.get("class_no"),
                number: r.get("number"),
                password_reset_required: r.get::<i64, _>("password_reset_required") != 0,
            })
            .collect(),
    ))
}

/// POST /api/classes/:id/students
pub async fn add_student(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(class_id): Path<i64>,
    Json(body): Json<AddStudentBody>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;
    require_class_access(teacher_id, class_id, &state.db).await?;

    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("ERR_STUDENT_NAME_REQUIRED".into()));
    }
    if body.grade < 1 || body.grade > 6 {
        return Err(ApiError::BadRequest("ERR_STUDENT_GRADE_INVALID".into()));
    }
    if body.class_no < 1 || body.class_no > 99 {
        return Err(ApiError::BadRequest("ERR_STUDENT_CLASS_NO_INVALID".into()));
    }
    if body.number < 1 || body.number > 99 {
        return Err(ApiError::BadRequest("ERR_STUDENT_NUMBER_INVALID".into()));
    }

    let username = format!("{}{:02}{:02}", body.grade, body.class_no, body.number);

    let mut tx = state.db.begin().await?;

    let insert_result = sqlx::query(
        "INSERT INTO students (username, name, grade, class_no, number) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&username)
    .bind(body.name.trim())
    .bind(body.grade)
    .bind(body.class_no)
    .bind(body.number)
    .execute(&mut *tx)
    .await;

    let student_id: i64 = match insert_result {
        Ok(r) => r.last_insert_rowid(),
        Err(e) => {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.message().contains("UNIQUE constraint failed") {
                    return Err(ApiError::BadRequest("ERR_STUDENT_ALREADY_EXISTS".into()));
                }
            }
            return Err(ApiError::Database(e));
        }
    };

    sqlx::query(
        "INSERT OR IGNORE INTO class_students (class_id, student_id) VALUES (?, ?)",
    )
    .bind(class_id)
    .bind(student_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(json!({ "id": student_id })))
}

/// POST /api/classes/:id/students/bulk
pub async fn bulk_add_students(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(class_id): Path<i64>,
    Json(body): Json<Vec<AddStudentBody>>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;
    require_class_access(teacher_id, class_id, &state.db).await?;

    if body.is_empty() {
        return Err(ApiError::BadRequest("ERR_BULK_EMPTY".into()));
    }

    let mut tx = state.db.begin().await?;
    let mut inserted: i64 = 0;
    let mut skipped: i64 = 0;

    for item in &body {
        if item.name.trim().is_empty() {
            continue;
        }
        let username = format!("{}{:02}{:02}", item.grade, item.class_no, item.number);

        let insert_result = sqlx::query(
            "INSERT INTO students (username, name, grade, class_no, number) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&username)
        .bind(item.name.trim())
        .bind(item.grade)
        .bind(item.class_no)
        .bind(item.number)
        .execute(&mut *tx)
        .await;

        let student_id: Option<i64> = match insert_result {
            Ok(r) => Some(r.last_insert_rowid()),
            Err(e) => {
                if let sqlx::Error::Database(ref db_err) = e {
                    if db_err.message().contains("UNIQUE constraint failed") {
                        sqlx::query_scalar(
                            "SELECT id FROM students WHERE grade = ? AND class_no = ? AND number = ?",
                        )
                        .bind(item.grade)
                        .bind(item.class_no)
                        .bind(item.number)
                        .fetch_optional(&mut *tx)
                        .await?
                    } else {
                        return Err(ApiError::Database(e));
                    }
                } else {
                    return Err(ApiError::Database(e));
                }
            }
        };

        if let Some(sid) = student_id {
            let result = sqlx::query(
                "INSERT OR IGNORE INTO class_students (class_id, student_id) VALUES (?, ?)",
            )
            .bind(class_id)
            .bind(sid)
            .execute(&mut *tx)
            .await?;
            if result.rows_affected() > 0 {
                inserted += 1;
            } else {
                skipped += 1;
            }
        }
    }

    tx.commit().await?;
    Ok(Json(json!({ "inserted": inserted, "skipped": skipped })))
}

// ── Import (strict rollback on any error) ────────────────────────────────────

#[derive(Deserialize)]
pub struct ImportStudentRow {
    pub grade: i64,
    pub class_no: i64,
    pub number: i64,
    pub name: String,
    pub username: Option<String>,
}

/// POST /api/classes/:id/students/import
pub async fn import_students(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(class_id): Path<i64>,
    Json(body): Json<Vec<ImportStudentRow>>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;
    require_class_access(teacher_id, class_id, &state.db).await?;

    if body.is_empty() {
        return Err(ApiError::BadRequest("ERR_IMPORT_EMPTY".into()));
    }

    let mut tx = state.db.begin().await?;

    for item in &body {
        if item.name.trim().is_empty() {
            return Err(ApiError::BadRequest("ERR_IMPORT_ROW_INVALID".into()));
        }
        let username = match &item.username {
            Some(u) if !u.trim().is_empty() => u.trim().to_string(),
            _ => format!("{}{:02}{:02}", item.grade, item.class_no, item.number),
        };

        let result = sqlx::query(
            "INSERT INTO students (username, name, grade, class_no, number) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&username)
        .bind(item.name.trim())
        .bind(item.grade)
        .bind(item.class_no)
        .bind(item.number)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.message().contains("UNIQUE constraint failed") {
                    return ApiError::BadRequest("ERR_IMPORT_DUPLICATE".into());
                }
            }
            ApiError::Database(e)
        })?;

        let student_id = result.last_insert_rowid();
        sqlx::query("INSERT INTO class_students (class_id, student_id) VALUES (?, ?)")
            .bind(class_id)
            .bind(student_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(Json(json!({ "imported": body.len() })))
}

/// DELETE /api/students/:id
pub async fn delete_student(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(student_id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;

    let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM students WHERE id = ?")
        .bind(student_id)
        .fetch_optional(&state.db)
        .await?;
    if exists.is_none() {
        return Err(ApiError::NotFound);
    }

    require_student_access(teacher_id, student_id, &state.db).await?;

    let mut tx = state.db.begin().await?;
    sqlx::query("DELETE FROM students WHERE id = ?")
        .bind(student_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok(Json(json!({ "ok": true })))
}

/// POST /api/students/:id/reset-password
pub async fn reset_student_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(student_id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;

    let username: Option<String> = sqlx::query_scalar("SELECT username FROM students WHERE id = ?")
        .bind(student_id)
        .fetch_optional(&state.db)
        .await?;
    let username = username.ok_or(ApiError::NotFound)?;

    require_student_access(teacher_id, student_id, &state.db).await?;

    // 해싱은 CPU 집약 작업이므로 트랜잭션 밖에서 수행 (커넥션 독점 방지)
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(username.as_bytes(), &salt)
        .map_err(|e| ApiError::Internal(format!("argon2: {e}")))?
        .to_string();

    let mut tx = state.db.begin().await?;
    sqlx::query(
        "UPDATE students SET password_hash = ?, password_reset_required = 1 WHERE id = ?",
    )
    .bind(&hash)
    .bind(student_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(Json(json!({ "ok": true })))
}
