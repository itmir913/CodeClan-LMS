use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};

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
    pub my_score: Option<i64>,
    pub total_max_score: i64,
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
    let (student_id, division_id, _) = parse_student_session(&state.db, &headers).await?;

    let rows = sqlx::query_as::<_, (i64, String, String, i64, Option<i64>, Option<String>, i64, Option<i64>, i64)>(
        r#"SELECT a.id, a.title, a.description,
                  (SELECT COUNT(*) FROM assessment_problems ap WHERE ap.assessment_id = a.id) AS problem_count,
                  (SELECT s.id   FROM sessions s WHERE s.assessment_id = a.id AND s.division_id = ? ORDER BY s.created_at DESC LIMIT 1) AS session_id,
                  (SELECT s.status FROM sessions s WHERE s.assessment_id = a.id AND s.division_id = ? ORDER BY s.created_at DESC LIMIT 1) AS session_status,
                  COALESCE(
                    (SELECT s.is_result_released FROM sessions s WHERE s.assessment_id = a.id AND s.division_id = ? ORDER BY s.created_at DESC LIMIT 1),
                    0
                  ) AS is_result_released,
                  (SELECT SUM(sub.score)
                   FROM sessions s2
                   JOIN submissions sub ON sub.session_id = s2.id AND sub.student_id = ? AND sub.is_latest = 1
                   WHERE s2.assessment_id = a.id AND s2.division_id = ? AND s2.status = 'CLOSED' AND s2.is_result_released = 1
                   ORDER BY s2.created_at DESC LIMIT 1) AS my_score,
                  COALESCE(
                    (SELECT SUM(ap2.score) FROM assessment_problems ap2 WHERE ap2.assessment_id = a.id),
                    0
                  ) AS total_max_score
           FROM assessments a
           JOIN assessment_divisions ad ON ad.assessment_id = a.id
           WHERE ad.division_id = ?
           ORDER BY a.created_at DESC"#,
    )
    .bind(division_id)
    .bind(division_id)
    .bind(division_id)
    .bind(student_id)
    .bind(division_id)
    .bind(division_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, title, description, problem_count, session_id, session_status, is_result_released, my_score, total_max_score)| {
                StudentAssessmentRow {
                    id,
                    title,
                    description,
                    problem_count,
                    session_id,
                    session_status,
                    is_result_released: is_result_released != 0,
                    my_score,
                    total_max_score,
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

// ─── GET /api/student/lessons/:id ─────────────────────────
// 내 분반에 공개된 차시 상세 (문항 포함)

#[derive(Serialize)]
pub struct StudentLessonProblem {
    pub id: i64,
    pub problem_id: i64,
    pub problem_type: i64,
    pub problem_title: String,
    pub description: String,
    pub type_config: String,
    pub order_no: i64,
}

#[derive(Serialize)]
pub struct StudentLessonDetail {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub order_no: i64,
    pub problems: Vec<StudentLessonProblem>,
}

pub async fn get_lesson_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(lesson_id): Path<i64>,
) -> Result<Json<StudentLessonDetail>, ApiError> {
    let (_, division_id, _) = parse_student_session(&state.db, &headers).await?;

    // 내 분반에 공개된 차시인지 확인
    let lesson = sqlx::query_as::<_, (i64, String, String, i64)>(
        r#"SELECT l.id, l.title, l.description, l.order_no
           FROM lessons l
           JOIN lesson_releases lr ON lr.lesson_id = l.id
           WHERE l.id = ? AND lr.division_id = ? AND lr.is_released = 1"#,
    )
    .bind(lesson_id)
    .bind(division_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    let problems = sqlx::query_as::<_, (i64, i64, i64, String, String, String, i64)>(
        r#"SELECT lp.id, lp.problem_id, p.type, p.title, p.description, p.type_config, lp.order_no
           FROM lesson_problems lp
           JOIN problems p ON p.id = lp.problem_id
           WHERE lp.lesson_id = ?
           ORDER BY lp.order_no"#,
    )
    .bind(lesson_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(StudentLessonDetail {
        id: lesson.0,
        title: lesson.1,
        description: lesson.2,
        order_no: lesson.3,
        problems: problems
            .into_iter()
            .map(|(id, problem_id, problem_type, problem_title, description, type_config, order_no)| {
                StudentLessonProblem { id, problem_id, problem_type, problem_title, description, type_config, order_no }
            })
            .collect(),
    }))
}
