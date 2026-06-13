use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, server::state::AppState};
use super::auth::parse_teacher_session;

// ─── 응답 타입 ─────────────────────────────────────────────

#[derive(Serialize)]
pub struct LessonRow {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub order_no: i64,
    pub problem_count: i64,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct LessonProblemRow {
    pub id: i64,
    pub problem_id: i64,
    pub problem_type: i64,
    pub problem_title: String,
    pub order_no: i64,
}

#[derive(Serialize)]
pub struct LessonDetail {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub order_no: i64,
    pub problems: Vec<LessonProblemRow>,
    pub releases: Vec<LessonRelease>,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct LessonRelease {
    pub division_id: i64,
    pub division_name: String,
    pub is_released: bool,
    pub released_at: Option<String>,
}

// ─── 차시 목록 ─────────────────────────────────────────────

pub async fn list_lessons(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<LessonRow>>, ApiError> {
    parse_teacher_session(&state.db, &headers).await?;

    let rows = sqlx::query_as::<_, (i64, String, String, i64, i64, String)>(
        r#"SELECT l.id, l.title, l.description, l.order_no,
                  (SELECT COUNT(*) FROM lesson_problems lp WHERE lp.lesson_id = l.id) AS problem_count,
                  l.created_at
           FROM lessons l
           ORDER BY l.order_no, l.created_at"#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, title, description, order_no, problem_count, created_at)| LessonRow {
                id, title, description, order_no, problem_count, created_at,
            })
            .collect(),
    ))
}

// ─── 차시 상세 ─────────────────────────────────────────────

pub async fn get_lesson(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<LessonDetail>, ApiError> {
    parse_teacher_session(&state.db, &headers).await?;

    let row = sqlx::query_as::<_, (i64, String, String, i64, String)>(
        "SELECT id, title, description, order_no, created_at FROM lessons WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    let problems = sqlx::query_as::<_, (i64, i64, i64, String, i64)>(
        r#"SELECT lp.id, lp.problem_id, p.type, p.title, lp.order_no
           FROM lesson_problems lp
           JOIN problems p ON p.id = lp.problem_id
           WHERE lp.lesson_id = ?
           ORDER BY lp.order_no"#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    let releases = sqlx::query_as::<_, (i64, String, i64, Option<String>)>(
        r#"SELECT lr.division_id, d.name, lr.is_released, lr.released_at
           FROM lesson_releases lr
           JOIN divisions d ON d.id = lr.division_id
           WHERE lr.lesson_id = ?
           ORDER BY d.name"#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(LessonDetail {
        id: row.0,
        title: row.1,
        description: row.2,
        order_no: row.3,
        created_at: row.4,
        problems: problems
            .into_iter()
            .map(|(id, problem_id, problem_type, problem_title, order_no)| LessonProblemRow {
                id, problem_id, problem_type, problem_title, order_no,
            })
            .collect(),
        releases: releases
            .into_iter()
            .map(|(division_id, division_name, is_released, released_at)| LessonRelease {
                division_id,
                division_name,
                is_released: is_released != 0,
                released_at,
            })
            .collect(),
    }))
}

// ─── 차시 생성 ─────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateLessonInput {
    pub title: String,
    pub description: Option<String>,
    pub order_no: Option<i64>,
}

pub async fn create_lesson(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreateLessonInput>,
) -> Result<Json<LessonRow>, ApiError> {
    let (teacher_id, _, _) = parse_teacher_session(&state.db, &headers).await?;

    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(ApiError::BadRequest("차시 제목을 입력하세요".into()));
    }

    let order_no = input.order_no.unwrap_or(0);
    let description = input.description.as_deref().unwrap_or("").to_string();

    let id = sqlx::query(
        "INSERT INTO lessons (title, description, order_no) VALUES (?, ?, ?)",
    )
    .bind(&title)
    .bind(&description)
    .bind(order_no)
    .execute(&state.db)
    .await?
    .last_insert_rowid();

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'lesson_create', 'lesson', ?, ?)",
    )
    .bind(teacher_id)
    .bind(id)
    .bind(format!("차시 생성: {title}"))
    .execute(&state.db)
    .await;

    Ok(Json(LessonRow {
        id, title, description, order_no, problem_count: 0,
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }))
}

// ─── 차시 수정 ─────────────────────────────────────────────

#[derive(Deserialize)]
pub struct UpdateLessonInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub order_no: Option<i64>,
}

pub async fn update_lesson(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<UpdateLessonInput>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (teacher_id, _, _) = parse_teacher_session(&state.db, &headers).await?;

    let exists: Option<(i64,)> = sqlx::query_as::<_, (i64,)>("SELECT id FROM lessons WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;
    if exists.is_none() { return Err(ApiError::NotFound); }

    if let Some(t) = &input.title {
        let t = t.trim();
        if !t.is_empty() {
            sqlx::query("UPDATE lessons SET title = ? WHERE id = ?").bind(t).bind(id).execute(&state.db).await?;
        }
    }
    if let Some(d) = &input.description {
        sqlx::query("UPDATE lessons SET description = ? WHERE id = ?").bind(d).bind(id).execute(&state.db).await?;
    }
    if let Some(n) = input.order_no {
        sqlx::query("UPDATE lessons SET order_no = ? WHERE id = ?").bind(n).bind(id).execute(&state.db).await?;
    }

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'lesson_update', 'lesson', ?, '차시 수정')",
    )
    .bind(teacher_id).bind(id).execute(&state.db).await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── 차시 삭제 ─────────────────────────────────────────────

pub async fn delete_lesson(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (teacher_id, _, _) = parse_teacher_session(&state.db, &headers).await?;

    let rows = sqlx::query("DELETE FROM lessons WHERE id = ?")
        .bind(id).execute(&state.db).await?.rows_affected();
    if rows == 0 { return Err(ApiError::NotFound); }

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'lesson_delete', 'lesson', ?, '차시 삭제')",
    )
    .bind(teacher_id).bind(id).execute(&state.db).await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── 차시 문항 배정 ────────────────────────────────────────

#[derive(Deserialize)]
pub struct SetLessonProblemsInput {
    pub problem_ids: Vec<i64>,
}

pub async fn set_lesson_problems(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<SetLessonProblemsInput>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (teacher_id, _, _) = parse_teacher_session(&state.db, &headers).await?;

    let exists: Option<(i64,)> = sqlx::query_as::<_, (i64,)>("SELECT id FROM lessons WHERE id = ?")
        .bind(id).fetch_optional(&state.db).await?;
    if exists.is_none() { return Err(ApiError::NotFound); }

    let mut tx = state.db.begin().await?;

    sqlx::query("DELETE FROM lesson_problems WHERE lesson_id = ?")
        .bind(id).execute(&mut *tx).await?;

    for (order, pid) in input.problem_ids.iter().enumerate() {
        sqlx::query("INSERT OR IGNORE INTO lesson_problems (lesson_id, problem_id, order_no) VALUES (?, ?, ?)")
            .bind(id).bind(pid).bind(order as i64).execute(&mut *tx).await?;
    }

    tx.commit().await?;

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'lesson_problems_update', 'lesson', ?, ?)",
    )
    .bind(teacher_id).bind(id)
    .bind(format!("문항 {}개 배정", input.problem_ids.len()))
    .execute(&state.db).await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── 분반별 공개 토글 ─────────────────────────────────────

#[derive(Deserialize)]
pub struct ToggleReleaseInput {
    pub division_id: i64,
    pub is_released: bool,
}

pub async fn toggle_release(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<ToggleReleaseInput>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (teacher_id, _, _) = parse_teacher_session(&state.db, &headers).await?;

    let released_at: Option<String> = if input.is_released {
        Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string())
    } else {
        None
    };

    sqlx::query(
        r#"INSERT INTO lesson_releases (lesson_id, division_id, is_released, released_at)
           VALUES (?, ?, ?, ?)
           ON CONFLICT(lesson_id, division_id) DO UPDATE SET
             is_released = excluded.is_released,
             released_at = excluded.released_at"#,
    )
    .bind(id)
    .bind(input.division_id)
    .bind(if input.is_released { 1i64 } else { 0i64 })
    .bind(&released_at)
    .execute(&state.db)
    .await?;

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'lesson_release_toggle', 'lesson', ?, ?)",
    )
    .bind(teacher_id).bind(id)
    .bind(if input.is_released { "차시 공개" } else { "차시 비공개" })
    .execute(&state.db).await;

    Ok(Json(serde_json::json!({ "ok": true, "released_at": released_at })))
}
