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

use crate::{error::ApiError, server::state::AppState};
use super::auth::parse_teacher_session;

#[derive(Serialize)]
pub struct TeacherRow {
    pub id: i64,
    pub username: String,
    pub name: String,
    pub role: String,
    pub division_count: i64,
    pub created_at: String,
}

// ─── 교사 목록 (admin only) ────────────────────────────────

pub async fn get_teachers(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<TeacherRow>>, ApiError> {
    let (_, _, role) = parse_teacher_session(&state.db, &headers).await?;
    if role != "admin" {
        return Err(ApiError::Forbidden);
    }

    let rows = sqlx::query_as::<_, (i64, String, String, String, i64, String)>(
        r#"SELECT t.id, t.username, t.name, t.role,
                  (SELECT COUNT(*) FROM teacher_divisions td WHERE td.teacher_id = t.id) AS division_count,
                  t.created_at
           FROM teachers t
           ORDER BY t.name"#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, username, name, role, division_count, created_at)| TeacherRow {
                id,
                username,
                name,
                role,
                division_count,
                created_at,
            })
            .collect(),
    ))
}

// ─── 교사 생성 (admin only) ────────────────────────────────

#[derive(Deserialize)]
pub struct CreateTeacherInput {
    pub username: String,
    pub name: String,
    pub password: String,
    pub role: Option<String>,
}

pub async fn create_teacher(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreateTeacherInput>,
) -> Result<Json<TeacherRow>, ApiError> {
    let (actor_id, _, role) = parse_teacher_session(&state.db, &headers).await?;
    if role != "admin" {
        return Err(ApiError::Forbidden);
    }

    let username = input.username.trim().to_string();
    let name = input.name.trim().to_string();
    let new_role = input.role.as_deref().unwrap_or("teacher");

    if username.is_empty() || name.is_empty() || input.password.is_empty() {
        return Err(ApiError::BadRequest("아이디, 이름, 비밀번호를 모두 입력하세요".into()));
    }

    if !matches!(new_role, "admin" | "teacher") {
        return Err(ApiError::BadRequest("role은 admin 또는 teacher여야 합니다".into()));
    }

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(input.password.as_bytes(), &salt)
        .map_err(|e| ApiError::InternalError(e.to_string()))?
        .to_string();

    let id = sqlx::query(
        "INSERT INTO teachers (username, name, password_hash, role) VALUES (?, ?, ?, ?)",
    )
    .bind(&username)
    .bind(&name)
    .bind(&hash)
    .bind(new_role)
    .execute(&state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            ApiError::BadRequest(format!("아이디 {username}은(는) 이미 사용 중입니다"))
        } else {
            ApiError::from(e)
        }
    })?
    .last_insert_rowid();

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'teacher_create', 'teacher', ?, ?)",
    )
    .bind(actor_id)
    .bind(id)
    .bind(format!("교사 계정 생성: {name} ({username}), 권한={new_role}"))
    .execute(&state.db)
    .await;

    Ok(Json(TeacherRow {
        id,
        username,
        name,
        role: new_role.to_string(),
        division_count: 0,
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }))
}

// ─── 교사 수정 (admin only) ────────────────────────────────

#[derive(Deserialize)]
pub struct UpdateTeacherInput {
    pub name: Option<String>,
    pub role: Option<String>,
    pub password: Option<String>,
}

pub async fn update_teacher(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<UpdateTeacherInput>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (actor_id, _, role) = parse_teacher_session(&state.db, &headers).await?;
    if role != "admin" {
        return Err(ApiError::Forbidden);
    }

    if let Some(new_role) = &input.role {
        if !matches!(new_role.as_str(), "admin" | "teacher") {
            return Err(ApiError::BadRequest("role은 admin 또는 teacher여야 합니다".into()));
        }
        sqlx::query("UPDATE teachers SET role = ? WHERE id = ?")
            .bind(new_role)
            .bind(id)
            .execute(&state.db)
            .await?;
    }

    if let Some(new_name) = &input.name {
        let new_name = new_name.trim();
        if !new_name.is_empty() {
            sqlx::query("UPDATE teachers SET name = ? WHERE id = ?")
                .bind(new_name)
                .bind(id)
                .execute(&state.db)
                .await?;
        }
    }

    if let Some(new_pw) = &input.password {
        if !new_pw.is_empty() {
            let salt = SaltString::generate(&mut OsRng);
            let hash = Argon2::default()
                .hash_password(new_pw.as_bytes(), &salt)
                .map_err(|e| ApiError::InternalError(e.to_string()))?
                .to_string();
            sqlx::query("UPDATE teachers SET password_hash = ? WHERE id = ?")
                .bind(&hash)
                .bind(id)
                .execute(&state.db)
                .await?;
        }
    }

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'teacher_update', 'teacher', ?, '교사 정보 수정')",
    )
    .bind(actor_id)
    .bind(id)
    .execute(&state.db)
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── 교사 삭제 (admin only) ────────────────────────────────

pub async fn delete_teacher(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (actor_id, _, role) = parse_teacher_session(&state.db, &headers).await?;
    if role != "admin" {
        return Err(ApiError::Forbidden);
    }

    if actor_id == id {
        return Err(ApiError::BadRequest("자신의 계정은 삭제할 수 없습니다".into()));
    }

    let rows = sqlx::query("DELETE FROM teachers WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(ApiError::NotFound);
    }

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'teacher_delete', 'teacher', ?, '교사 계정 삭제')",
    )
    .bind(actor_id)
    .bind(id)
    .execute(&state.db)
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}
