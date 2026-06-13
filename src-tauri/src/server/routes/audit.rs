use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, server::state::AppState};
use super::auth::parse_teacher_session;

#[derive(Deserialize)]
pub struct AuditQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub action_type: Option<String>,
}

#[derive(Serialize)]
pub struct AuditLogRow {
    pub id: i64,
    pub actor_teacher_id: Option<i64>,
    pub actor_name: Option<String>,
    pub action_type: String,
    pub target_type: Option<String>,
    pub target_id: Option<i64>,
    pub detail: Option<String>,
    pub created_at: String,
}

/// GET /api/audit-logs
pub async fn list_audit_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<AuditQuery>,
) -> Result<Json<Vec<AuditLogRow>>, ApiError> {
    parse_teacher_session(&state.db, &headers).await?;

    let limit = q.limit.unwrap_or(100).min(500);
    let offset = q.offset.unwrap_or(0);

    let rows = if let Some(ref action_type) = q.action_type {
        sqlx::query_as::<_, (i64, Option<i64>, Option<String>, String, Option<String>, Option<i64>, Option<String>, String)>(
            r#"SELECT al.id, al.actor_teacher_id, t.name,
                      al.action_type, al.target_type, al.target_id, al.detail, al.created_at
               FROM audit_logs al
               LEFT JOIN teachers t ON t.id = al.actor_teacher_id
               WHERE al.action_type = ?
               ORDER BY al.created_at DESC
               LIMIT ? OFFSET ?"#,
        )
        .bind(action_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, (i64, Option<i64>, Option<String>, String, Option<String>, Option<i64>, Option<String>, String)>(
            r#"SELECT al.id, al.actor_teacher_id, t.name,
                      al.action_type, al.target_type, al.target_id, al.detail, al.created_at
               FROM audit_logs al
               LEFT JOIN teachers t ON t.id = al.actor_teacher_id
               ORDER BY al.created_at DESC
               LIMIT ? OFFSET ?"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(
        rows.into_iter()
            .map(|(id, actor_teacher_id, actor_name, action_type, target_type, target_id, detail, created_at)| {
                AuditLogRow { id, actor_teacher_id, actor_name, action_type, target_type, target_id, detail, created_at }
            })
            .collect(),
    ))
}
