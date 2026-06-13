use axum::{
    extract::State,
    http::HeaderMap,
    Json,
};
use serde::Serialize;

use crate::{error::ApiError, server::state::AppState};
use super::auth::parse_student_session;

// ─── 응답 타입 ─────────────────────────────────────────────

#[derive(Serialize)]
pub struct StudentLessonRow {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub order_no: i64,
    pub problem_count: i64,
    pub released_at: Option<String>,
}

#[derive(Serialize)]
pub struct StudentAssessmentRow {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub problem_count: i64,
    pub session_id: Option<i64>,
    pub session_status: Option<String>,
    pub is_result_released: bool,
}

#[derive(Serialize)]
pub struct StudentActiveSession {
    pub id: i64,
    pub assessment_id: i64,
    pub assessment_title: String,
    pub status: String,
    pub time_limit_min: Option<i64>,
    pub start_at: Option<String>,
    pub is_paused: bool,
    pub is_result_released: bool,
}

// ─── GET /api/student/lessons ──────────────────────────────
// 내 분반에 공개된 차시 목록

pub async fn get_lessons(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<StudentLessonRow>>, ApiError> {
    let (_, division_id, _) = parse_student_session(&state.db, &headers).await?;

    let rows = sqlx::query_as::<_, (i64, String, String, i64, i64, Option<String>)>(
        r#"SELECT l.id, l.title, l.description, l.order_no,
                  (SELECT COUNT(*) FROM lesson_problems lp WHERE lp.lesson_id = l.id) AS problem_count,
                  lr.released_at
           FROM lessons l
           JOIN lesson_releases lr ON lr.lesson_id = l.id
           WHERE lr.division_id = ? AND lr.is_released = 1
           ORDER BY l.order_no, l.id"#,
    )
    .bind(division_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, title, description, order_no, problem_count, released_at)| {
                StudentLessonRow { id, title, description, order_no, problem_count, released_at }
            })
            .collect(),
    ))
}

// ─── GET /api/student/assessments ─────────────────────────
// 내 분반에 연결된 수행평가 목록 (최신 세션 상태 포함)

pub async fn get_assessments(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<StudentAssessmentRow>>, ApiError> {
    let (_, division_id, _) = parse_student_session(&state.db, &headers).await?;

    let rows = sqlx::query_as::<_, (i64, String, String, i64, Option<i64>, Option<String>, i64)>(
        r#"SELECT a.id, a.title, a.description,
                  (SELECT COUNT(*) FROM assessment_problems ap WHERE ap.assessment_id = a.id) AS problem_count,
                  (SELECT s.id   FROM sessions s WHERE s.assessment_id = a.id AND s.division_id = ? ORDER BY s.created_at DESC LIMIT 1) AS session_id,
                  (SELECT s.status FROM sessions s WHERE s.assessment_id = a.id AND s.division_id = ? ORDER BY s.created_at DESC LIMIT 1) AS session_status,
                  COALESCE(
                    (SELECT s.is_result_released FROM sessions s WHERE s.assessment_id = a.id AND s.division_id = ? ORDER BY s.created_at DESC LIMIT 1),
                    0
                  ) AS is_result_released
           FROM assessments a
           JOIN assessment_divisions ad ON ad.assessment_id = a.id
           WHERE ad.division_id = ?
           ORDER BY a.created_at DESC"#,
    )
    .bind(division_id)
    .bind(division_id)
    .bind(division_id)
    .bind(division_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, title, description, problem_count, session_id, session_status, is_result_released)| {
                StudentAssessmentRow {
                    id,
                    title,
                    description,
                    problem_count,
                    session_id,
                    session_status,
                    is_result_released: is_result_released != 0,
                }
            })
            .collect(),
    ))
}

// ─── GET /api/student/active-session ──────────────────────
// 현재 분반의 LOBBY 또는 RUNNING 세션 (없으면 null)

pub async fn get_active_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Option<StudentActiveSession>>, ApiError> {
    let (_, division_id, _) = parse_student_session(&state.db, &headers).await?;

    let row = sqlx::query_as::<_, (i64, i64, String, String, Option<i64>, Option<String>, i64, i64)>(
        r#"SELECT s.id, s.assessment_id, a.title, s.status,
                  s.time_limit_min, s.start_at, s.is_paused, s.is_result_released
           FROM sessions s
           JOIN assessments a ON a.id = s.assessment_id
           WHERE s.division_id = ? AND s.status IN ('LOBBY', 'RUNNING')
           ORDER BY s.created_at DESC
           LIMIT 1"#,
    )
    .bind(division_id)
    .fetch_optional(&state.db)
    .await?;

    Ok(Json(row.map(
        |(id, assessment_id, assessment_title, status, time_limit_min, start_at, is_paused, is_result_released)| {
            StudentActiveSession {
                id,
                assessment_id,
                assessment_title,
                status,
                time_limit_min,
                start_at,
                is_paused: is_paused != 0,
                is_result_released: is_result_released != 0,
            }
        },
    )))
}
