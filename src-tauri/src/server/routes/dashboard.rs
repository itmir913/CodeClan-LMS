use axum::{extract::State, http::HeaderMap, Json};
use serde::Serialize;

use crate::{error::ApiError, server::state::AppState};
use super::auth::parse_teacher_session;

#[derive(Serialize)]
pub struct ActiveSession {
    pub id: i64,
    pub assessment_title: String,
    pub status: String,
    pub submission_count: i64,
    pub student_count: i64,
    pub time_limit_min: Option<i64>,
    pub start_at: Option<String>,
}

#[derive(Serialize)]
pub struct DivisionCard {
    pub id: i64,
    pub name: String,
    pub student_count: i64,
    pub active_session: Option<ActiveSession>,
}

#[derive(Serialize)]
pub struct DashboardStats {
    pub problem_count: i64,
    pub lesson_count: i64,
    pub assessment_count: i64,
}

#[derive(Serialize)]
pub struct RecentLog {
    pub id: i64,
    pub action_type: String,
    pub detail: Option<String>,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct DashboardResponse {
    pub teacher_name: String,
    pub teacher_role: String,
    pub division_count: i64,
    pub divisions: Vec<DivisionCard>,
    pub stats: DashboardStats,
    pub recent_logs: Vec<RecentLog>,
}

/// GET /api/dashboard
pub async fn get_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DashboardResponse>, ApiError> {
    let (teacher_id, teacher_name, teacher_role) = parse_teacher_session(&state.db, &headers).await?;

    // admin은 전체 분반, 일반 교사는 담당 분반만
    let divisions: Vec<(i64, String)> = if teacher_role == "admin" {
        sqlx::query_as::<_, (i64, String)>("SELECT id, name FROM divisions ORDER BY name")
            .fetch_all(&state.db)
            .await?
    } else {
        sqlx::query_as::<_, (i64, String)>(
            r#"SELECT d.id, d.name
               FROM divisions d
               JOIN teacher_divisions td ON td.division_id = d.id
               WHERE td.teacher_id = ?
               ORDER BY d.name"#,
        )
        .bind(teacher_id)
        .fetch_all(&state.db)
        .await?
    };

    let mut division_cards: Vec<DivisionCard> = Vec::new();

    for (div_id, div_name) in &divisions {
        let student_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM students WHERE division_id = ?",
        )
        .bind(div_id)
        .fetch_one(&state.db)
        .await?;

        // RUNNING 또는 LOBBY 상태의 세션 (가장 최근 것)
        let active_session = sqlx::query_as::<_, (i64, String, String, Option<i64>, Option<String>)>(
            r#"SELECT s.id, a.title, s.status, s.time_limit_min, s.start_at
               FROM sessions s
               JOIN assessments a ON a.id = s.assessment_id
               WHERE s.division_id = ? AND s.status IN ('RUNNING', 'LOBBY')
               ORDER BY s.created_at DESC
               LIMIT 1"#,
        )
        .bind(div_id)
        .fetch_optional(&state.db)
        .await?;

        let active = if let Some((sid, title, status, time_limit, start_at)) = active_session {
            let submission_count = sqlx::query_scalar::<_, i64>(
                r#"SELECT COUNT(DISTINCT student_id)
                   FROM submissions
                   WHERE session_id = ? AND is_latest = 1"#,
            )
            .bind(sid)
            .fetch_one(&state.db)
            .await?;

            Some(ActiveSession {
                id: sid,
                assessment_title: title,
                status,
                submission_count,
                student_count,
                time_limit_min: time_limit,
                start_at,
            })
        } else {
            None
        };

        division_cards.push(DivisionCard {
            id: *div_id,
            name: div_name.clone(),
            student_count,
            active_session: active,
        });
    }

    // 전역 통계
    let problem_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM problems")
        .fetch_one(&state.db)
        .await?;
    let lesson_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM lessons")
        .fetch_one(&state.db)
        .await?;
    let assessment_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM assessments")
        .fetch_one(&state.db)
        .await?;

    // 최근 감사 로그 5개
    let recent_logs = sqlx::query_as::<_, (i64, String, Option<String>, String)>(
        "SELECT id, action_type, detail, created_at FROM audit_logs ORDER BY created_at DESC LIMIT 5",
    )
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|(id, action_type, detail, created_at)| RecentLog {
        id,
        action_type,
        detail,
        created_at,
    })
    .collect();

    Ok(Json(DashboardResponse {
        teacher_name,
        teacher_role,
        division_count: divisions.len() as i64,
        divisions: division_cards,
        stats: DashboardStats {
            problem_count,
            lesson_count,
            assessment_count,
        },
        recent_logs,
    }))
}
