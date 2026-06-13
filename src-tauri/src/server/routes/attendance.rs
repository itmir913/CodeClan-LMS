use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AttendanceQuery {
    pub division_id: Option<i64>,
}

use crate::{error::ApiError, server::state::AppState};
use super::auth::{parse_student_session, parse_teacher_session};

// ─── 학생 하트비트 ─────────────────────────────────────────

#[derive(Deserialize)]
pub struct HeartbeatInput {
    pub context_type: String,
    pub context_id: i64,
}

/// POST /api/student/heartbeat
/// 학생 접속 중 주기적으로 호출해 last_seen_at 갱신
pub async fn student_heartbeat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<HeartbeatInput>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (student_id, _, _) = parse_student_session(&state.db, &headers).await?;

    if !matches!(body.context_type.as_str(), "lesson" | "session") {
        return Err(ApiError::BadRequest(
            "context_type은 lesson 또는 session이어야 합니다".into(),
        ));
    }

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // 기존 행 UPDATE 시도
    let affected = sqlx::query(
        "UPDATE attendance_heartbeats SET last_seen_at = ? \
         WHERE student_id = ? AND context_type = ? AND context_id = ?",
    )
    .bind(&now)
    .bind(student_id)
    .bind(&body.context_type)
    .bind(body.context_id)
    .execute(&state.db)
    .await?
    .rows_affected();

    // 최초 접속 → INSERT (is_late 판단 포함)
    if affected == 0 {
        let is_late = if body.context_type == "session" {
            sqlx::query_scalar::<_, Option<String>>(
                "SELECT start_at FROM sessions WHERE id = ?",
            )
            .bind(body.context_id)
            .fetch_optional(&state.db)
            .await?
            .flatten()
            .is_some()
        } else {
            false
        };

        sqlx::query(
            r#"INSERT INTO attendance_heartbeats
               (student_id, context_type, context_id, joined_at, last_seen_at, is_late)
               VALUES (?, ?, ?, ?, ?, ?)"#,
        )
        .bind(student_id)
        .bind(&body.context_type)
        .bind(body.context_id)
        .bind(&now)
        .bind(&now)
        .bind(is_late)
        .execute(&state.db)
        .await?;
    }

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── 교사: 세션 출결 현황 ───────────────────────────────────

#[derive(Serialize)]
pub struct AttendanceRow {
    pub student_id: i64,
    pub name: String,
    pub student_number: String,
    pub is_online: bool,
    pub joined_at: Option<String>,
    pub last_seen_at: Option<String>,
    pub is_late: bool,
}

/// GET /api/sessions/:id/attendance
/// 세션 분반 학생 전체의 실시간 접속 현황 (30초 이내 = 온라인)
pub async fn get_session_attendance(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<Vec<AttendanceRow>>, ApiError> {
    let (teacher_id, _, role) = parse_teacher_session(&state.db, &headers).await?;

    let accessible = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM sessions WHERE id = ? AND (? = 'admin' OR division_id IN (
            SELECT division_id FROM teacher_divisions WHERE teacher_id = ?
        ))"#,
    )
    .bind(session_id)
    .bind(&role)
    .bind(teacher_id)
    .fetch_one(&state.db)
    .await?;

    if accessible == 0 {
        return Err(ApiError::Forbidden);
    }

    let division_id: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT division_id FROM sessions WHERE id = ?",
    )
    .bind(session_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    let rows = sqlx::query_as::<_, (i64, String, String, Option<String>, Option<String>, Option<i64>)>(
        r#"SELECT s.id, s.name, s.student_number,
                  ah.joined_at, ah.last_seen_at, ah.is_late
           FROM students s
           LEFT JOIN attendance_heartbeats ah
             ON ah.student_id = s.id
             AND ah.context_type = 'session'
             AND ah.context_id = ?
           WHERE s.division_id = ?
           ORDER BY s.student_number"#,
    )
    .bind(session_id)
    .bind(division_id)
    .fetch_all(&state.db)
    .await?;

    let now_ts = chrono::Utc::now().timestamp();
    const ONLINE_THRESHOLD_SECS: i64 = 30;

    Ok(Json(
        rows.into_iter()
            .map(|(id, name, student_number, joined_at, last_seen_at, is_late)| {
                let is_online = last_seen_at.as_deref().map(|ts| {
                    chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S")
                        .ok()
                        .map(|dt| now_ts - dt.and_utc().timestamp() <= ONLINE_THRESHOLD_SECS)
                        .unwrap_or(false)
                }).unwrap_or(false);

                AttendanceRow {
                    student_id: id,
                    name,
                    student_number,
                    is_online,
                    joined_at,
                    last_seen_at,
                    is_late: is_late.unwrap_or(0) != 0,
                }
            })
            .collect(),
    ))
}

// ─── 교사: 차시 출결 현황 ───────────────────────────────────

/// GET /api/lessons/:id/attendance?division_id=X
pub async fn get_lesson_attendance(
    State(state): State<AppState>,
    Path(lesson_id): Path<i64>,
    Query(q): Query<AttendanceQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<AttendanceRow>>, ApiError> {
    parse_teacher_session(&state.db, &headers).await?;

    let division_id = q.division_id.ok_or(ApiError::BadRequest("division_id required".into()))?;

    let rows = sqlx::query_as::<_, (i64, String, String, Option<String>, Option<String>, Option<i64>)>(
        r#"SELECT s.id, s.name, s.student_number,
                  ah.joined_at, ah.last_seen_at, ah.is_late
           FROM students s
           LEFT JOIN attendance_heartbeats ah
             ON ah.student_id = s.id
             AND ah.context_type = 'lesson'
             AND ah.context_id = ?
           WHERE s.division_id = ?
           ORDER BY s.student_number"#,
    )
    .bind(lesson_id)
    .bind(division_id)
    .fetch_all(&state.db)
    .await?;

    let now_ts = chrono::Utc::now().timestamp();
    const ONLINE_THRESHOLD_SECS: i64 = 30;

    Ok(Json(
        rows.into_iter()
            .map(|(id, name, student_number, joined_at, last_seen_at, is_late)| {
                let is_online = last_seen_at.as_deref().map(|ts| {
                    chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S")
                        .ok()
                        .map(|dt| now_ts - dt.and_utc().timestamp() <= ONLINE_THRESHOLD_SECS)
                        .unwrap_or(false)
                }).unwrap_or(false);
                AttendanceRow {
                    student_id: id,
                    name,
                    student_number,
                    is_online,
                    joined_at,
                    last_seen_at,
                    is_late: is_late.unwrap_or(0) != 0,
                }
            })
            .collect(),
    ))
}
