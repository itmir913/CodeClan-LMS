use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, server::state::AppState};
use super::auth::parse_teacher_session;

// ─── 응답 타입 ─────────────────────────────────────────────

#[derive(Serialize)]
pub struct SessionRow {
    pub id: i64,
    pub assessment_id: i64,
    pub assessment_title: String,
    pub division_id: i64,
    pub division_name: String,
    pub status: String,
    pub target_type: String,
    pub time_limit_min: Option<i64>,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub is_paused: bool,
    pub is_result_released: bool,
    pub submission_count: i64,
    pub student_count: i64,
    pub created_at: String,
}

// ─── 세션 목록 ─────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SessionListQuery {
    pub division_id: Option<i64>,
    pub assessment_id: Option<i64>,
    pub status: Option<String>,
}

pub async fn list_sessions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SessionListQuery>,
) -> Result<Json<Vec<SessionRow>>, ApiError> {
    let (teacher_id, _, role) = parse_teacher_session(&state.db, &headers).await?;

    let rows = sqlx::query_as::<_, (i64, i64, String, i64, String, String, String, Option<i64>, Option<String>, Option<String>, i64, i64, i64, i64, String)>(
        r#"SELECT s.id, s.assessment_id, a.title, s.division_id, d.name,
                  s.status, s.target_type, s.time_limit_min, s.start_at, s.end_at,
                  s.is_paused, s.is_result_released,
                  (SELECT COUNT(*) FROM submissions sub WHERE sub.session_id = s.id AND sub.is_latest = 1) AS submission_count,
                  (SELECT COUNT(*) FROM students st WHERE st.division_id = s.division_id) AS student_count,
                  s.created_at
           FROM sessions s
           JOIN assessments a ON a.id = s.assessment_id
           JOIN divisions d ON d.id = s.division_id
           WHERE (? IS NULL OR s.division_id = ?)
             AND (? IS NULL OR s.assessment_id = ?)
             AND (? IS NULL OR s.status = ?)
             AND (? = 'admin' OR s.division_id IN (
               SELECT td.division_id FROM teacher_divisions td WHERE td.teacher_id = ?
             ))
           ORDER BY s.created_at DESC"#,
    )
    .bind(query.division_id)
    .bind(query.division_id)
    .bind(query.assessment_id)
    .bind(query.assessment_id)
    .bind(query.status.as_deref())
    .bind(query.status.as_deref())
    .bind(&role)
    .bind(teacher_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, assessment_id, assessment_title, division_id, division_name,
                   status, target_type, time_limit_min, start_at, end_at,
                   is_paused, is_result_released, submission_count, student_count, created_at)| SessionRow {
                id, assessment_id, assessment_title, division_id, division_name,
                status, target_type, time_limit_min, start_at, end_at,
                is_paused: is_paused != 0,
                is_result_released: is_result_released != 0,
                submission_count, student_count, created_at,
            })
            .collect(),
    ))
}

// ─── 세션 생성 (CREATED 상태로) ────────────────────────────

#[derive(Deserialize)]
pub struct CreateSessionInput {
    pub assessment_id: i64,
    pub division_id: i64,
    pub target_type: Option<String>,
    pub time_limit_min: Option<i64>,
    pub student_ids: Option<Vec<i64>>,
}

pub async fn create_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreateSessionInput>,
) -> Result<Json<SessionRow>, ApiError> {
    let (teacher_id, _, role) = parse_teacher_session(&state.db, &headers).await?;

    // 담당 분반 확인 (admin은 제외)
    if role != "admin" {
        let ok: Option<(i64,)> = sqlx::query_as::<_, (i64,)>(
            "SELECT division_id FROM teacher_divisions WHERE teacher_id = ? AND division_id = ?",
        )
        .bind(teacher_id)
        .bind(input.division_id)
        .fetch_optional(&state.db)
        .await?;
        if ok.is_none() { return Err(ApiError::Forbidden); }
    }

    // 같은 분반에 LOBBY 또는 RUNNING 세션이 이미 있으면 거부
    let active: i64 = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM sessions WHERE division_id = ? AND status IN ('LOBBY', 'RUNNING')",
    )
    .bind(input.division_id)
    .fetch_one(&state.db)
    .await?
    .0;

    if active > 0 {
        return Err(ApiError::BadRequest("이미 활성화된 세션이 있는 분반입니다".into()));
    }

    let target_type = input.target_type.as_deref().unwrap_or("ALL").to_uppercase();
    if !matches!(target_type.as_str(), "ALL" | "INDIVIDUAL") {
        return Err(ApiError::BadRequest("target_type은 ALL 또는 INDIVIDUAL이어야 합니다".into()));
    }

    let mut tx = state.db.begin().await?;

    let session_id = sqlx::query(
        r#"INSERT INTO sessions (assessment_id, division_id, target_type, time_limit_min, status)
           VALUES (?, ?, ?, ?, 'CREATED')"#,
    )
    .bind(input.assessment_id)
    .bind(input.division_id)
    .bind(&target_type)
    .bind(input.time_limit_min)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    // INDIVIDUAL 타입이면 session_targets 삽입
    if target_type == "INDIVIDUAL" {
        if let Some(student_ids) = &input.student_ids {
            for sid in student_ids {
                sqlx::query("INSERT OR IGNORE INTO session_targets (session_id, student_id) VALUES (?, ?)")
                    .bind(session_id).bind(sid).execute(&mut *tx).await?;
            }
        }
    }

    tx.commit().await?;

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'session_create', 'session', ?, ?)",
    )
    .bind(teacher_id).bind(session_id)
    .bind(format!("세션 생성 (분반 {}, 평가 {})", input.division_id, input.assessment_id))
    .execute(&state.db).await;

    // 학생 수 조회
    let student_count: i64 = if target_type == "ALL" {
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM students WHERE division_id = ?")
            .bind(input.division_id).fetch_one(&state.db).await?.0
    } else {
        input.student_ids.as_ref().map(|v| v.len() as i64).unwrap_or(0)
    };

    let assessment_title: String = sqlx::query_as::<_, (String,)>("SELECT title FROM assessments WHERE id = ?")
        .bind(input.assessment_id).fetch_one(&state.db).await?.0;

    let division_name: String = sqlx::query_as::<_, (String,)>("SELECT name FROM divisions WHERE id = ?")
        .bind(input.division_id).fetch_one(&state.db).await?.0;

    Ok(Json(SessionRow {
        id: session_id,
        assessment_id: input.assessment_id,
        assessment_title,
        division_id: input.division_id,
        division_name,
        status: "CREATED".into(),
        target_type,
        time_limit_min: input.time_limit_min,
        start_at: None,
        end_at: None,
        is_paused: false,
        is_result_released: false,
        submission_count: 0,
        student_count,
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }))
}

// ─── 세션 상태 전환 ────────────────────────────────────────
// CREATED ⇄ LOBBY → RUNNING → CLOSED

#[derive(Deserialize)]
pub struct TransitionInput {
    pub action: String,
}

pub async fn transition_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<TransitionInput>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (teacher_id, _, _) = parse_teacher_session(&state.db, &headers).await?;

    let (current_status,): (String,) = sqlx::query_as::<_, (String,)>(
        "SELECT status FROM sessions WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    let (new_status, set_start): (&str, bool) = match (current_status.as_str(), input.action.as_str()) {
        ("CREATED", "to_lobby") => ("LOBBY", false),
        ("LOBBY", "to_created") => ("CREATED", false),
        ("LOBBY", "to_running") => ("RUNNING", true),
        ("RUNNING", "close") => ("CLOSED", false),
        _ => {
            return Err(ApiError::BadRequest(format!(
                "현재 상태 '{current_status}'에서 '{}' 전환은 허용되지 않습니다", input.action
            )));
        }
    };

    if set_start {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        sqlx::query("UPDATE sessions SET status = ?, start_at = ? WHERE id = ?")
            .bind(new_status).bind(&now).bind(id).execute(&state.db).await?;
    } else {
        sqlx::query("UPDATE sessions SET status = ? WHERE id = ?")
            .bind(new_status).bind(id).execute(&state.db).await?;
    }

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'session_transition', 'session', ?, ?)",
    )
    .bind(teacher_id).bind(id)
    .bind(format!("{} → {}", current_status, new_status))
    .execute(&state.db).await;

    Ok(Json(serde_json::json!({ "ok": true, "status": new_status })))
}

// ─── 일시정지 / 재개 ───────────────────────────────────────

pub async fn pause_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (teacher_id, _, _) = parse_teacher_session(&state.db, &headers).await?;

    let (status, is_paused): (String, i64) = sqlx::query_as::<_, (String, i64)>(
        "SELECT status, is_paused FROM sessions WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    if status != "RUNNING" {
        return Err(ApiError::BadRequest("RUNNING 세션만 일시정지/재개할 수 있습니다".into()));
    }

    let new_paused = if is_paused == 0 { 1i64 } else { 0i64 };
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    sqlx::query("UPDATE sessions SET is_paused = ?, paused_at = ? WHERE id = ?")
        .bind(new_paused)
        .bind(if new_paused == 1 { Some(now) } else { None })
        .bind(id)
        .execute(&state.db)
        .await?;

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'session_pause_toggle', 'session', ?, ?)",
    )
    .bind(teacher_id).bind(id)
    .bind(if new_paused == 1 { "세션 일시정지" } else { "세션 재개" })
    .execute(&state.db).await;

    Ok(Json(serde_json::json!({ "ok": true, "is_paused": new_paused == 1 })))
}

// ─── 결과 공개 토글 ────────────────────────────────────────

pub async fn toggle_result_release(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (teacher_id, _, _) = parse_teacher_session(&state.db, &headers).await?;

    let (status, is_released): (String, i64) = sqlx::query_as::<_, (String, i64)>(
        "SELECT status, is_result_released FROM sessions WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    if status != "CLOSED" {
        return Err(ApiError::BadRequest("CLOSED 세션만 결과를 공개/비공개할 수 있습니다".into()));
    }

    let new_val = if is_released == 0 { 1i64 } else { 0i64 };
    sqlx::query("UPDATE sessions SET is_result_released = ? WHERE id = ?")
        .bind(new_val).bind(id).execute(&state.db).await?;

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'session_result_release', 'session', ?, ?)",
    )
    .bind(teacher_id).bind(id)
    .bind(if new_val == 1 { "결과 공개" } else { "결과 비공개" })
    .execute(&state.db).await;

    Ok(Json(serde_json::json!({ "ok": true, "is_result_released": new_val == 1 })))
}
