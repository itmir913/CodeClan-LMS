use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{error::ApiError, server::state::AppState};
use super::auth::{parse_student_session, parse_teacher_session};

// ─── 학생 전용 ─────────────────────────────────────────────

#[derive(Serialize)]
pub struct SessionProblemRow {
    pub ap_id: i64,
    pub order_no: i64,
    pub max_score: i64,
    pub problem_id: i64,
    pub problem_type: i64,
    pub title: String,
    pub description: String,
    pub type_config: String,
    pub is_structure_check: bool,
    pub submission_id: Option<i64>,
    pub submitted_content: Option<String>,
    pub submitted_language: Option<String>,
    pub verdict: Option<String>,
    pub submitted_score: Option<i64>,
}

/// GET /api/student/session-problems
/// 현재 RUNNING 세션의 문제 목록 + 학생 기제출 내역
pub async fn get_session_problems(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<SessionProblemRow>>, ApiError> {
    let (student_id, division_id, _) = parse_student_session(&state.db, &headers).await?;

    let session = sqlx::query_as::<_, (i64, i64)>(
        r#"SELECT id, assessment_id FROM sessions
           WHERE division_id = ?
             AND (status = 'RUNNING' OR (status = 'CLOSED' AND is_result_released = 1))
           ORDER BY created_at DESC LIMIT 1"#,
    )
    .bind(division_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    let (session_id, assessment_id) = session;

    let rows = sqlx::query_as::<_, (i64, i64, i64, i64, i64, String, String, String, i64, Option<i64>, Option<String>, Option<String>, Option<String>, Option<i64>)>(
        r#"SELECT ap.id, ap.order_no, ap.score,
                  p.id, p.type, p.title, p.description, p.type_config, p.is_structure_check,
                  sub.id, sub.content, sub.language, sub.verdict, sub.score
           FROM assessment_problems ap
           JOIN problems p ON p.id = ap.problem_id
           LEFT JOIN submissions sub ON sub.problem_id = p.id
               AND sub.student_id = ? AND sub.session_id = ? AND sub.is_latest = 1
           WHERE ap.assessment_id = ?
           ORDER BY ap.order_no"#,
    )
    .bind(student_id)
    .bind(session_id)
    .bind(assessment_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|(ap_id, order_no, max_score, problem_id, problem_type, title, description, type_config, is_sc, sub_id, sub_content, sub_lang, verdict, sub_score)| {
                SessionProblemRow {
                    ap_id,
                    order_no,
                    max_score,
                    problem_id,
                    problem_type,
                    title,
                    description,
                    type_config,
                    is_structure_check: is_sc != 0,
                    submission_id: sub_id,
                    submitted_content: sub_content,
                    submitted_language: sub_lang,
                    verdict,
                    submitted_score: sub_score,
                }
            })
            .collect(),
    ))
}

/// GET /api/student/sessions/:id/problems
/// 특정 CLOSED+released 세션의 문제 목록 + 내 답안 + 점수 (결과 조회용)
pub async fn get_session_result_problems(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(session_id): Path<i64>,
) -> Result<Json<Vec<SessionProblemRow>>, ApiError> {
    let (student_id, division_id, _) = parse_student_session(&state.db, &headers).await?;

    let assessment_id = sqlx::query_scalar::<_, i64>(
        r#"SELECT assessment_id FROM sessions
           WHERE id = ? AND division_id = ? AND status = 'CLOSED' AND is_result_released = 1"#,
    )
    .bind(session_id)
    .bind(division_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    let rows = sqlx::query_as::<_, (i64, i64, i64, i64, i64, String, String, String, i64, Option<i64>, Option<String>, Option<String>, Option<String>, Option<i64>)>(
        r#"SELECT ap.id, ap.order_no, ap.score,
                  p.id, p.type, p.title, p.description, p.type_config, p.is_structure_check,
                  sub.id, sub.content, sub.language, sub.verdict, sub.score
           FROM assessment_problems ap
           JOIN problems p ON p.id = ap.problem_id
           LEFT JOIN submissions sub ON sub.problem_id = p.id
               AND sub.student_id = ? AND sub.session_id = ? AND sub.is_latest = 1
           WHERE ap.assessment_id = ?
           ORDER BY ap.order_no"#,
    )
    .bind(student_id)
    .bind(session_id)
    .bind(assessment_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|(ap_id, order_no, max_score, problem_id, problem_type, title, description, type_config, is_sc, sub_id, sub_content, sub_lang, verdict, sub_score)| {
                SessionProblemRow {
                    ap_id, order_no, max_score, problem_id, problem_type,
                    title, description, type_config,
                    is_structure_check: is_sc != 0,
                    submission_id: sub_id,
                    submitted_content: sub_content,
                    submitted_language: sub_lang,
                    verdict,
                    submitted_score: sub_score,
                }
            })
            .collect(),
    ))
}

#[derive(Deserialize)]
pub struct SubmitRequest {
    pub problem_id: i64,
    pub content: String,
    pub language: Option<String>,
}

#[derive(Serialize)]
pub struct SubmissionResult {
    pub submission_id: i64,
    pub verdict: Option<String>,
    pub score: Option<i64>,
}

/// POST /api/student/submit
/// 활성 RUNNING 세션에 답안 제출; 유형①는 expected_output 비교로 자동채점
pub async fn submit_answer(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<SubmitRequest>,
) -> Result<Json<SubmissionResult>, ApiError> {
    let (student_id, division_id, _) = parse_student_session(&state.db, &headers).await?;

    let session_row = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT id, assessment_id, is_paused FROM sessions WHERE division_id = ? AND status = 'RUNNING' ORDER BY created_at DESC LIMIT 1",
    )
    .bind(division_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::BadRequest("진행 중인 시험 세션이 없습니다".into()))?;

    let (session_id, assessment_id, is_paused) = session_row;

    if is_paused != 0 {
        return Err(ApiError::BadRequest("세션이 일시정지 중입니다".into()));
    }

    // 문제가 이 수행평가에 속하는지 확인
    let problem_row = sqlx::query_as::<_, (i64, i64, String)>(
        r#"SELECT p.type, ap.score, p.type_config
           FROM assessment_problems ap
           JOIN problems p ON p.id = ap.problem_id
           WHERE ap.problem_id = ? AND ap.assessment_id = ?"#,
    )
    .bind(body.problem_id)
    .bind(assessment_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::BadRequest("이 시험에 포함되지 않은 문제입니다".into()))?;

    let (problem_type, max_score, type_config) = problem_row;

    // 유형별 채점
    let (verdict, score): (Option<String>, Option<i64>) = match problem_type {
        1 => {
            let cfg: Value = serde_json::from_str(&type_config).unwrap_or(Value::Null);
            let expected = cfg.get("expected_output").and_then(|v| v.as_str()).unwrap_or("");
            if expected.is_empty() {
                (None, None) // 기대값 미설정 → 수동채점
            } else if body.content.trim() == expected.trim() {
                (Some("AC".into()), Some(max_score))
            } else {
                (Some("WA".into()), Some(0))
            }
        }
        2 => (Some("PENDING".into()), None), // wbox 미연동
        _ => (None, None),                   // 수동채점
    };

    let judged_at = verdict.as_deref().map(|_| {
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
    });

    let mut tx = state.db.begin().await?;

    let prev_no = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(MAX(submission_no), 0) FROM submissions WHERE student_id = ? AND problem_id = ? AND session_id = ?",
    )
    .bind(student_id)
    .bind(body.problem_id)
    .bind(session_id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE submissions SET is_latest = 0 WHERE student_id = ? AND problem_id = ? AND session_id = ? AND is_latest = 1",
    )
    .bind(student_id)
    .bind(body.problem_id)
    .bind(session_id)
    .execute(&mut *tx)
    .await?;

    let sub_id = sqlx::query_scalar::<_, i64>(
        r#"INSERT INTO submissions (problem_id, student_id, session_id, submission_no, is_latest, language, content, verdict, score, judged_at)
           VALUES (?, ?, ?, ?, 1, ?, ?, ?, ?, ?)
           RETURNING id"#,
    )
    .bind(body.problem_id)
    .bind(student_id)
    .bind(session_id)
    .bind(prev_no + 1)
    .bind(&body.language)
    .bind(&body.content)
    .bind(&verdict)
    .bind(score)
    .bind(&judged_at)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(SubmissionResult { submission_id: sub_id, verdict, score }))
}

// ─── 교사 전용 ─────────────────────────────────────────────

#[derive(Serialize)]
pub struct SubmissionRow {
    pub id: i64,
    pub student_id: i64,
    pub student_name: String,
    pub student_number: String,
    pub problem_id: i64,
    pub problem_type: i64,
    pub problem_title: String,
    pub problem_order: i64,
    pub max_score: i64,
    pub content: String,
    pub language: Option<String>,
    pub verdict: Option<String>,
    pub score: Option<i64>,
    pub submission_no: i64,
    pub created_at: String,
}

/// GET /api/sessions/:id/submissions
/// 세션의 모든 제출 목록 (교사용)
pub async fn get_session_submissions(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<Vec<SubmissionRow>>, ApiError> {
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

    let rows = sqlx::query_as::<_, (i64, i64, String, String, i64, i64, String, i64, i64, String, Option<String>, Option<String>, Option<i64>, i64, String)>(
        r#"SELECT sub.id, sub.student_id, st.name, st.student_number,
                  sub.problem_id, p.type, p.title, ap.order_no, ap.score,
                  sub.content, sub.language, sub.verdict, sub.score,
                  sub.submission_no, sub.created_at
           FROM submissions sub
           JOIN students st ON st.id = sub.student_id
           JOIN problems p ON p.id = sub.problem_id
           JOIN sessions sess ON sess.id = sub.session_id
           JOIN assessment_problems ap ON ap.problem_id = p.id AND ap.assessment_id = sess.assessment_id
           WHERE sub.session_id = ? AND sub.is_latest = 1
           ORDER BY st.student_number, ap.order_no"#,
    )
    .bind(session_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, student_id, student_name, student_number, problem_id, problem_type, problem_title, problem_order, max_score, content, language, verdict, score, submission_no, created_at)| {
                SubmissionRow {
                    id, student_id, student_name, student_number,
                    problem_id, problem_type, problem_title, problem_order, max_score,
                    content, language, verdict, score, submission_no, created_at,
                }
            })
            .collect(),
    ))
}

#[derive(Deserialize)]
pub struct GradeRequest {
    pub score: i64,
}

/// POST /api/submissions/:id/grade
/// 수동 채점 (교사용)
pub async fn grade_submission(
    State(state): State<AppState>,
    Path(submission_id): Path<i64>,
    headers: HeaderMap,
    Json(body): Json<GradeRequest>,
) -> Result<Json<Value>, ApiError> {
    let (teacher_id, _, role) = parse_teacher_session(&state.db, &headers).await?;

    let accessible = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM submissions sub
           JOIN sessions s ON s.id = sub.session_id
           WHERE sub.id = ? AND (? = 'admin' OR s.division_id IN (
               SELECT division_id FROM teacher_divisions WHERE teacher_id = ?
           ))"#,
    )
    .bind(submission_id)
    .bind(&role)
    .bind(teacher_id)
    .fetch_one(&state.db)
    .await?;

    if accessible == 0 {
        return Err(ApiError::Forbidden);
    }

    let max_score: i64 = sqlx::query_scalar::<_, i64>(
        r#"SELECT ap.score
           FROM assessment_problems ap
           JOIN sessions s ON s.assessment_id = ap.assessment_id
           JOIN submissions sub ON sub.problem_id = ap.problem_id AND sub.session_id = s.id
           WHERE sub.id = ?"#,
    )
    .bind(submission_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    if body.score < 0 || body.score > max_score {
        return Err(ApiError::BadRequest(format!(
            "점수는 0 이상 {} 이하이어야 합니다", max_score
        )));
    }

    sqlx::query(
        "UPDATE submissions SET score = ?, verdict = 'GRADED', judged_at = datetime('now') WHERE id = ?",
    )
    .bind(body.score)
    .bind(submission_id)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({ "ok": true })))
}
