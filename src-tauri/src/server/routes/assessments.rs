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
pub struct AssessmentRow {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub problem_count: i64,
    pub division_count: i64,
    pub is_locked: bool,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct AssessmentProblemRow {
    pub id: i64,
    pub problem_id: i64,
    pub problem_type: i64,
    pub problem_title: String,
    pub order_no: i64,
    pub score: i64,
}

#[derive(Serialize)]
pub struct AssessmentDivisionRow {
    pub division_id: i64,
    pub division_name: String,
    pub has_running_session: bool,
}

#[derive(Serialize)]
pub struct AssessmentDetail {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub is_locked: bool,
    pub problems: Vec<AssessmentProblemRow>,
    pub divisions: Vec<AssessmentDivisionRow>,
    pub created_at: String,
}

// ─── 수행평가 목록 ─────────────────────────────────────────

pub async fn list_assessments(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<AssessmentRow>>, ApiError> {
    parse_teacher_session(&state.db, &headers).await?;

    let rows = sqlx::query_as::<_, (i64, String, String, i64, i64, String)>(
        r#"SELECT a.id, a.title, a.description,
                  (SELECT COUNT(*) FROM assessment_problems ap WHERE ap.assessment_id = a.id) AS problem_count,
                  (SELECT COUNT(*) FROM assessment_divisions ad WHERE ad.assessment_id = a.id) AS division_count,
                  a.created_at
           FROM assessments a
           ORDER BY a.created_at DESC"#,
    )
    .fetch_all(&state.db)
    .await?;

    // RUNNING 세션이 하나라도 있으면 is_locked = true
    let mut result = Vec::new();
    for (id, title, description, problem_count, division_count, created_at) in rows {
        let running: i64 = sqlx::query_as::<_, (i64,)>(
            r#"SELECT COUNT(*) FROM sessions s
               JOIN assessment_divisions ad ON ad.division_id = s.division_id
               WHERE ad.assessment_id = ? AND s.assessment_id = ? AND s.status = 'RUNNING'"#,
        )
        .bind(id)
        .bind(id)
        .fetch_one(&state.db)
        .await?
        .0;

        result.push(AssessmentRow {
            id, title, description, problem_count, division_count,
            is_locked: running > 0,
            created_at,
        });
    }

    Ok(Json(result))
}

// ─── 수행평가 상세 ─────────────────────────────────────────

pub async fn get_assessment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<AssessmentDetail>, ApiError> {
    parse_teacher_session(&state.db, &headers).await?;

    let row = sqlx::query_as::<_, (i64, String, String, String)>(
        "SELECT id, title, description, created_at FROM assessments WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    let problems = sqlx::query_as::<_, (i64, i64, i64, String, i64, i64)>(
        r#"SELECT ap.id, ap.problem_id, p.type, p.title, ap.order_no, ap.score
           FROM assessment_problems ap
           JOIN problems p ON p.id = ap.problem_id
           WHERE ap.assessment_id = ?
           ORDER BY ap.order_no"#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    let divisions = sqlx::query_as::<_, (i64, String)>(
        r#"SELECT ad.division_id, d.name
           FROM assessment_divisions ad
           JOIN divisions d ON d.id = ad.division_id
           WHERE ad.assessment_id = ?
           ORDER BY d.name"#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    // 분반별 RUNNING 세션 여부
    let mut division_rows = Vec::new();
    for (division_id, division_name) in divisions {
        let running: i64 = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM sessions WHERE assessment_id = ? AND division_id = ? AND status = 'RUNNING'",
        )
        .bind(id)
        .bind(division_id)
        .fetch_one(&state.db)
        .await?
        .0;

        division_rows.push(AssessmentDivisionRow {
            division_id,
            division_name,
            has_running_session: running > 0,
        });
    }

    let is_locked = division_rows.iter().any(|d| d.has_running_session);

    Ok(Json(AssessmentDetail {
        id: row.0,
        title: row.1,
        description: row.2,
        is_locked,
        created_at: row.3,
        problems: problems
            .into_iter()
            .map(|(id, problem_id, problem_type, problem_title, order_no, score)| AssessmentProblemRow {
                id, problem_id, problem_type, problem_title, order_no, score,
            })
            .collect(),
        divisions: division_rows,
    }))
}

// ─── 수행평가 생성 ─────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateAssessmentInput {
    pub title: String,
    pub description: Option<String>,
}

pub async fn create_assessment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreateAssessmentInput>,
) -> Result<Json<AssessmentRow>, ApiError> {
    let (teacher_id, _, _) = parse_teacher_session(&state.db, &headers).await?;

    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(ApiError::BadRequest("수행평가 제목을 입력하세요".into()));
    }

    let description = input.description.as_deref().unwrap_or("").to_string();

    let id = sqlx::query("INSERT INTO assessments (title, description) VALUES (?, ?)")
        .bind(&title)
        .bind(&description)
        .execute(&state.db)
        .await?
        .last_insert_rowid();

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'assessment_create', 'assessment', ?, ?)",
    )
    .bind(teacher_id).bind(id).bind(format!("수행평가 생성: {title}"))
    .execute(&state.db).await;

    Ok(Json(AssessmentRow {
        id, title, description, problem_count: 0, division_count: 0, is_locked: false,
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }))
}

// ─── 수행평가 수정 (잠금 체크) ────────────────────────────

#[derive(Deserialize)]
pub struct UpdateAssessmentInput {
    pub title: Option<String>,
    pub description: Option<String>,
}

pub async fn update_assessment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<UpdateAssessmentInput>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (teacher_id, _, _) = parse_teacher_session(&state.db, &headers).await?;

    ensure_not_locked(&state.db, id).await?;

    if let Some(t) = &input.title {
        let t = t.trim();
        if !t.is_empty() {
            sqlx::query("UPDATE assessments SET title = ? WHERE id = ?").bind(t).bind(id).execute(&state.db).await?;
        }
    }
    if let Some(d) = &input.description {
        sqlx::query("UPDATE assessments SET description = ? WHERE id = ?").bind(d).bind(id).execute(&state.db).await?;
    }

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'assessment_update', 'assessment', ?, '수행평가 수정')",
    )
    .bind(teacher_id).bind(id).execute(&state.db).await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── 수행평가 삭제 ─────────────────────────────────────────

pub async fn delete_assessment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (teacher_id, _, _) = parse_teacher_session(&state.db, &headers).await?;

    ensure_not_locked(&state.db, id).await?;

    let rows = sqlx::query("DELETE FROM assessments WHERE id = ?")
        .bind(id).execute(&state.db).await?.rows_affected();
    if rows == 0 { return Err(ApiError::NotFound); }

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'assessment_delete', 'assessment', ?, '수행평가 삭제')",
    )
    .bind(teacher_id).bind(id).execute(&state.db).await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── 문항 배점 배정 (잠금 체크) ──────────────────────────

#[derive(Deserialize)]
pub struct AssessmentProblemItem {
    pub problem_id: i64,
    pub score: i64,
}

pub async fn set_assessment_problems(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<Vec<AssessmentProblemItem>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (teacher_id, _, _) = parse_teacher_session(&state.db, &headers).await?;

    ensure_not_locked(&state.db, id).await?;

    let mut tx = state.db.begin().await?;

    sqlx::query("DELETE FROM assessment_problems WHERE assessment_id = ?")
        .bind(id).execute(&mut *tx).await?;

    for (order, item) in input.iter().enumerate() {
        sqlx::query("INSERT OR IGNORE INTO assessment_problems (assessment_id, problem_id, order_no, score) VALUES (?, ?, ?, ?)")
            .bind(id).bind(item.problem_id).bind(order as i64).bind(item.score)
            .execute(&mut *tx).await?;
    }

    tx.commit().await?;

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'assessment_problems_update', 'assessment', ?, ?)",
    )
    .bind(teacher_id).bind(id)
    .bind(format!("문항 {}개 배정", input.len()))
    .execute(&state.db).await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── 분반 연결/해제 ────────────────────────────────────────

#[derive(Deserialize)]
pub struct LinkDivisionInput {
    pub division_id: i64,
}

pub async fn link_division(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<LinkDivisionInput>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (teacher_id, _, _) = parse_teacher_session(&state.db, &headers).await?;

    sqlx::query("INSERT OR IGNORE INTO assessment_divisions (assessment_id, division_id) VALUES (?, ?)")
        .bind(id).bind(input.division_id).execute(&state.db).await?;

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'assessment_division_link', 'assessment', ?, ?)",
    )
    .bind(teacher_id).bind(id).bind(format!("분반 {} 연결", input.division_id))
    .execute(&state.db).await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn unlink_division(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((id, division_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (teacher_id, _, _) = parse_teacher_session(&state.db, &headers).await?;

    // RUNNING 세션이 있으면 해제 불가
    let running: i64 = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM sessions WHERE assessment_id = ? AND division_id = ? AND status = 'RUNNING'",
    )
    .bind(id).bind(division_id)
    .fetch_one(&state.db).await?.0;

    if running > 0 {
        return Err(ApiError::BadRequest("RUNNING 상태인 세션이 있는 분반은 연결 해제할 수 없습니다".into()));
    }

    sqlx::query("DELETE FROM assessment_divisions WHERE assessment_id = ? AND division_id = ?")
        .bind(id).bind(division_id).execute(&state.db).await?;

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'assessment_division_unlink', 'assessment', ?, ?)",
    )
    .bind(teacher_id).bind(id).bind(format!("분반 {} 연결 해제", division_id))
    .execute(&state.db).await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── 헬퍼: 편집 잠금 확인 ─────────────────────────────────

async fn ensure_not_locked(db: &sqlx::SqlitePool, assessment_id: i64) -> Result<(), ApiError> {
    let running: i64 = sqlx::query_as::<_, (i64,)>(
        r#"SELECT COUNT(*) FROM sessions s
           JOIN assessment_divisions ad ON ad.division_id = s.division_id
           WHERE ad.assessment_id = ? AND s.assessment_id = ? AND s.status = 'RUNNING'"#,
    )
    .bind(assessment_id)
    .bind(assessment_id)
    .fetch_one(db)
    .await?
    .0;

    if running > 0 {
        Err(ApiError::BadRequest(
            "RUNNING 상태인 세션이 있는 수행평가는 편집할 수 없습니다".into(),
        ))
    } else {
        Ok(())
    }
}
