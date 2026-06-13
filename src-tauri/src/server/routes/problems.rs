use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, server::state::AppState};
use super::auth::parse_teacher_session;

// ─── 타입 상수 ────────────────────────────────────────────

pub const TYPE_OUTPUT_MATCH: i64 = 1;
pub const TYPE_CODE_JUDGE: i64 = 2;
pub const TYPE_REPORT: i64 = 3;
pub const TYPE_FILL_BLANK: i64 = 4;

// ─── 응답 타입 ─────────────────────────────────────────────

#[derive(Serialize)]
pub struct ProblemRow {
    pub id: i64,
    pub problem_type: i64,
    pub title: String,
    pub description: String,
    pub type_config: String,
    pub is_structure_check: bool,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct ProblemListItem {
    pub id: i64,
    pub problem_type: i64,
    pub title: String,
    pub is_structure_check: bool,
    pub created_at: String,
}

// ─── 문제 목록 ─────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListQuery {
    pub problem_type: Option<i64>,
    pub q: Option<String>,
}

pub async fn list_problems(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<ProblemListItem>>, ApiError> {
    parse_teacher_session(&state.db, &headers).await?;

    let type_filter = query.problem_type;
    let search = query.q.as_deref().map(|s| format!("%{s}%"));

    let rows = sqlx::query_as::<_, (i64, i64, String, i64, String)>(
        r#"SELECT id, type, title, is_structure_check, created_at
           FROM problems
           WHERE (? IS NULL OR type = ?)
             AND (? IS NULL OR title LIKE ? OR description LIKE ?)
           ORDER BY created_at DESC"#,
    )
    .bind(type_filter)
    .bind(type_filter)
    .bind(search.clone())
    .bind(search.clone())
    .bind(search)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, problem_type, title, isc, created_at)| ProblemListItem {
                id,
                problem_type,
                title,
                is_structure_check: isc != 0,
                created_at,
            })
            .collect(),
    ))
}

// ─── 문제 단건 조회 ────────────────────────────────────────

pub async fn get_problem(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<ProblemRow>, ApiError> {
    parse_teacher_session(&state.db, &headers).await?;

    let row = sqlx::query_as::<_, (i64, i64, String, String, String, i64, String)>(
        "SELECT id, type, title, description, type_config, is_structure_check, created_at FROM problems WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(ProblemRow {
        id: row.0,
        problem_type: row.1,
        title: row.2,
        description: row.3,
        type_config: row.4,
        is_structure_check: row.5 != 0,
        created_at: row.6,
    }))
}

// ─── 문제 생성 ─────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateProblemInput {
    pub problem_type: i64,
    pub title: String,
    pub description: Option<String>,
    pub type_config: Option<String>,
    pub is_structure_check: Option<bool>,
}

pub async fn create_problem(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreateProblemInput>,
) -> Result<Json<ProblemRow>, ApiError> {
    let (teacher_id, _, _) = parse_teacher_session(&state.db, &headers).await?;

    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(ApiError::BadRequest("문제 제목을 입력하세요".into()));
    }

    if !(1..=4).contains(&input.problem_type) {
        return Err(ApiError::BadRequest("문항 유형은 1~4이어야 합니다".into()));
    }

    // type_config JSON 유효성 검사
    let type_config = input.type_config.as_deref().unwrap_or("{}");
    if serde_json::from_str::<serde_json::Value>(type_config).is_err() {
        return Err(ApiError::BadRequest("type_config가 유효한 JSON이 아닙니다".into()));
    }

    let description = input.description.as_deref().unwrap_or("").to_string();
    let is_structure_check = input.is_structure_check.unwrap_or(false);

    let id = sqlx::query(
        r#"INSERT INTO problems (type, title, description, type_config, is_structure_check)
           VALUES (?, ?, ?, ?, ?)"#,
    )
    .bind(input.problem_type)
    .bind(&title)
    .bind(&description)
    .bind(type_config)
    .bind(if is_structure_check { 1i64 } else { 0i64 })
    .execute(&state.db)
    .await?
    .last_insert_rowid();

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'problem_create', 'problem', ?, ?)",
    )
    .bind(teacher_id)
    .bind(id)
    .bind(format!("[유형{}] {title}", input.problem_type))
    .execute(&state.db)
    .await;

    Ok(Json(ProblemRow {
        id,
        problem_type: input.problem_type,
        title,
        description,
        type_config: type_config.to_string(),
        is_structure_check,
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }))
}

// ─── 문제 수정 ─────────────────────────────────────────────

#[derive(Deserialize)]
pub struct UpdateProblemInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub type_config: Option<String>,
    pub is_structure_check: Option<bool>,
}

pub async fn update_problem(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<UpdateProblemInput>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (teacher_id, _, _) = parse_teacher_session(&state.db, &headers).await?;

    // 문제 존재 확인
    let exists: Option<(i64,)> = sqlx::query_as::<_, (i64,)>("SELECT id FROM problems WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;
    if exists.is_none() {
        return Err(ApiError::NotFound);
    }

    if let Some(title) = &input.title {
        let title = title.trim();
        if !title.is_empty() {
            sqlx::query("UPDATE problems SET title = ? WHERE id = ?")
                .bind(title)
                .bind(id)
                .execute(&state.db)
                .await?;
        }
    }

    if let Some(desc) = &input.description {
        sqlx::query("UPDATE problems SET description = ? WHERE id = ?")
            .bind(desc)
            .bind(id)
            .execute(&state.db)
            .await?;
    }

    if let Some(tc) = &input.type_config {
        if serde_json::from_str::<serde_json::Value>(tc).is_err() {
            return Err(ApiError::BadRequest("type_config가 유효한 JSON이 아닙니다".into()));
        }
        sqlx::query("UPDATE problems SET type_config = ? WHERE id = ?")
            .bind(tc)
            .bind(id)
            .execute(&state.db)
            .await?;
    }

    if let Some(isc) = input.is_structure_check {
        sqlx::query("UPDATE problems SET is_structure_check = ? WHERE id = ?")
            .bind(if isc { 1i64 } else { 0i64 })
            .bind(id)
            .execute(&state.db)
            .await?;
    }

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'problem_update', 'problem', ?, '문제 수정')",
    )
    .bind(teacher_id)
    .bind(id)
    .execute(&state.db)
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── 문제 삭제 ─────────────────────────────────────────────

pub async fn delete_problem(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (teacher_id, _, _) = parse_teacher_session(&state.db, &headers).await?;

    // 배정된 차시/수행평가가 있으면 삭제 불가
    let lesson_count: i64 =
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM lesson_problems WHERE problem_id = ?")
            .bind(id)
            .fetch_one(&state.db)
            .await?
            .0;

    let assessment_count: i64 =
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM assessment_problems WHERE problem_id = ?")
            .bind(id)
            .fetch_one(&state.db)
            .await?
            .0;

    if lesson_count > 0 || assessment_count > 0 {
        return Err(ApiError::BadRequest(
            "차시 또는 수행평가에 배정된 문제는 삭제할 수 없습니다. 먼저 배정을 해제하세요.".into(),
        ));
    }

    let rows = sqlx::query("DELETE FROM problems WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(ApiError::NotFound);
    }

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'problem_delete', 'problem', ?, '문제 삭제')",
    )
    .bind(teacher_id)
    .bind(id)
    .execute(&state.db)
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}
