use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::{
    error::ApiError,
    judge::normalize_tc_text,
    server::{routes::auth::parse_session, state::AppState},
};

// ── Request types ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ProblemListQuery {
    #[serde(rename = "type")]
    pub type_slug: Option<String>,
}

#[derive(Deserialize)]
pub struct ChoiceInput {
    pub content: String,
    pub is_correct: bool,
}

#[derive(Deserialize)]
pub struct TestCaseInput {
    pub input: String,
    pub expected_output: String,
    #[serde(default)]
    pub is_sample: bool,
    #[serde(default)]
    pub explanation: String,
}

#[derive(Deserialize)]
pub struct ProblemBody {
    #[serde(rename = "type")]
    pub type_slug: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub is_draft: bool,
    pub subject_id: Option<i64>,
    // short_answer
    pub answer: Option<String>,
    #[serde(default)]
    pub case_sensitive: bool,
    // multiple_choice
    #[serde(default)]
    pub allow_multiple: bool,
    pub choices: Option<Vec<ChoiceInput>>,
    // code_submit
    #[serde(default)]
    pub input_format: String,
    #[serde(default)]
    pub output_format: String,
    #[serde(default)]
    pub constraints: String,
    #[serde(default = "default_time_limit")]
    pub time_limit_ms: i64,
    #[serde(default = "default_memory_limit")]
    pub memory_limit_mb: i64,
    #[serde(default = "default_show_io")]
    pub show_io_on_fail: bool,
    pub test_cases: Option<Vec<TestCaseInput>>,
}

fn default_time_limit() -> i64 {
    1000
}
fn default_memory_limit() -> i64 {
    128
}
fn default_show_io() -> bool {
    true
}

// ── Response types ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ProblemListItem {
    pub id: i64,
    pub uuid: String,
    #[serde(rename = "type")]
    pub type_slug: String,
    pub title: String,
    pub subject_id: Option<i64>,
    pub subject_name: Option<String>,
    pub is_draft: bool,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct Choice {
    pub id: i64,
    pub order_no: i64,
    pub content: String,
    pub is_correct: bool,
}

#[derive(Serialize)]
pub struct TestCaseDetail {
    pub id: i64,
    pub number: i64,
    pub input: String,
    pub expected_output: String,
    pub is_sample: bool,
    pub explanation: String,
}

#[derive(Serialize)]
pub struct ProblemDetail {
    pub id: i64,
    pub uuid: String,
    #[serde(rename = "type")]
    pub type_slug: String,
    pub title: String,
    pub description: String,
    pub comment: String,
    pub subject_id: Option<i64>,
    pub subject_name: Option<String>,
    pub is_draft: bool,
    pub created_at: String,
    // short_answer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub case_sensitive: Option<bool>,
    // multiple_choice
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_multiple: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<Choice>>,
    // code_submit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_limit_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_limit_mb: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_io_on_fail: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_cases: Option<Vec<TestCaseDetail>>,
}

// ── File helpers ───────────────────────────────────────────────────────────────

fn problem_dir(data_dir: &PathBuf, problem_id: i64) -> PathBuf {
    data_dir.join("problems").join(problem_id.to_string())
}

async fn read_tc_content(path: &std::path::Path) -> String {
    tokio::fs::read_to_string(path).await.unwrap_or_default()
}

async fn write_tc_files_to_dir(
    dir: &std::path::Path,
    test_cases: &[TestCaseInput],
) -> Result<(), ApiError> {
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|e| ApiError::Internal(format!("tc dir create: {e}")))?;
    for (i, tc) in test_cases.iter().enumerate() {
        let n = i as i64 + 1;
        tokio::fs::write(dir.join(format!("{n}.in")), normalize_tc_text(&tc.input))
            .await
            .map_err(|e| ApiError::Internal(format!("tc write: {e}")))?;
        tokio::fs::write(dir.join(format!("{n}.out")), normalize_tc_text(&tc.expected_output))
            .await
            .map_err(|e| ApiError::Internal(format!("tc write: {e}")))?;
    }
    Ok(())
}

async fn write_tc_files(
    data_dir: &PathBuf,
    problem_id: i64,
    test_cases: &[TestCaseInput],
) -> Result<(), ApiError> {
    let dir = problem_dir(data_dir, problem_id);
    write_tc_files_to_dir(&dir, test_cases).await
}

async fn delete_tc_dir(data_dir: &PathBuf, problem_id: i64) -> Result<(), ApiError> {
    let dir = problem_dir(data_dir, problem_id);
    if dir.exists() {
        tokio::fs::remove_dir_all(&dir)
            .await
            .map_err(|e| ApiError::Internal(format!("tc dir remove: {e}")))?;
    }
    Ok(())
}

// ── DB helpers ─────────────────────────────────────────────────────────────────

async fn type_id_from_slug(slug: &str, db: &sqlx::SqlitePool) -> Result<i64, ApiError> {
    let id: Option<i64> = sqlx::query_scalar("SELECT id FROM problem_types WHERE slug = ?")
        .bind(slug)
        .fetch_optional(db)
        .await?;
    id.ok_or_else(|| ApiError::BadRequest("ERR_PROBLEM_TYPE_INVALID".into()))
}

fn validate_body(body: &ProblemBody) -> Result<(), ApiError> {
    if body.title.trim().is_empty() {
        return Err(ApiError::BadRequest("ERR_PROBLEM_TITLE_REQUIRED".into()));
    }
    match body.type_slug.as_str() {
        "short_answer" => {
            if body
                .answer
                .as_deref()
                .map(|s| s.trim().is_empty())
                .unwrap_or(true)
            {
                return Err(ApiError::BadRequest("ERR_PROBLEM_ANSWER_REQUIRED".into()));
            }
        }
        "multiple_choice" => {
            let choices = body.choices.as_deref().unwrap_or(&[]);
            if choices.len() < 2 {
                return Err(ApiError::BadRequest("ERR_PROBLEM_CHOICES_MIN".into()));
            }
            if !choices.iter().any(|c| c.is_correct) {
                return Err(ApiError::BadRequest("ERR_PROBLEM_ANSWER_REQUIRED".into()));
            }
        }
        "code_submit" => {}
        _ => return Err(ApiError::BadRequest("ERR_PROBLEM_TYPE_INVALID".into())),
    }
    Ok(())
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/problems
pub async fn list_problems(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<ProblemListQuery>,
) -> Result<Json<Vec<ProblemListItem>>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    session.teacher_id.ok_or(ApiError::Forbidden)?;

    use sqlx::Row as _;

    let rows = if let Some(ref slug) = params.type_slug {
        sqlx::query(
            "SELECT p.id, p.uuid, pt.slug AS type_slug, p.title, p.subject_id, \
                    s.name AS subject_name, p.is_draft, p.created_at \
             FROM problems p \
             JOIN problem_types pt ON pt.id = p.type_id \
             LEFT JOIN subjects s ON s.id = p.subject_id \
             WHERE pt.slug = ? \
             ORDER BY p.created_at DESC",
        )
        .bind(slug)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query(
            "SELECT p.id, p.uuid, pt.slug AS type_slug, p.title, p.subject_id, \
                    s.name AS subject_name, p.is_draft, p.created_at \
             FROM problems p \
             JOIN problem_types pt ON pt.id = p.type_id \
             LEFT JOIN subjects s ON s.id = p.subject_id \
             ORDER BY p.created_at DESC",
        )
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(
        rows.iter()
            .map(|r| ProblemListItem {
                id: r.get("id"),
                uuid: r.get("uuid"),
                type_slug: r.get("type_slug"),
                title: r.get("title"),
                subject_id: r.get("subject_id"),
                subject_name: r.get("subject_name"),
                is_draft: r.get::<i64, _>("is_draft") != 0,
                created_at: r.get("created_at"),
            })
            .collect(),
    ))
}

/// GET /api/problems/:id
pub async fn get_problem(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(problem_id): Path<i64>,
) -> Result<Json<ProblemDetail>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    session.teacher_id.ok_or(ApiError::Forbidden)?;

    use sqlx::Row as _;

    let row = sqlx::query(
        "SELECT p.id, p.uuid, pt.slug AS type_slug, p.title, p.description, p.comment, \
                p.subject_id, s.name AS subject_name, p.is_draft, p.created_at \
         FROM problems p \
         JOIN problem_types pt ON pt.id = p.type_id \
         LEFT JOIN subjects s ON s.id = p.subject_id \
         WHERE p.id = ?",
    )
    .bind(problem_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    let type_slug: String = row.get("type_slug");

    let mut detail = ProblemDetail {
        id: row.get("id"),
        uuid: row.get("uuid"),
        type_slug: type_slug.clone(),
        title: row.get("title"),
        description: row.get("description"),
        comment: row.get("comment"),
        subject_id: row.get("subject_id"),
        subject_name: row.get("subject_name"),
        is_draft: row.get::<i64, _>("is_draft") != 0,
        created_at: row.get("created_at"),
        answer: None,
        case_sensitive: None,
        allow_multiple: None,
        choices: None,
        input_format: None,
        output_format: None,
        constraints: None,
        time_limit_ms: None,
        memory_limit_mb: None,
        show_io_on_fail: None,
        test_cases: None,
    };

    match type_slug.as_str() {
        "short_answer" => {
            if let Some(r) = sqlx::query(
                "SELECT answer, case_sensitive FROM problem_short_answers WHERE problem_id = ?",
            )
            .bind(problem_id)
            .fetch_optional(&state.db)
            .await?
            {
                detail.answer = Some(r.get("answer"));
                detail.case_sensitive = Some(r.get::<i64, _>("case_sensitive") != 0);
            }
        }
        "multiple_choice" => {
            if let Some(r) = sqlx::query(
                "SELECT allow_multiple FROM problem_multiple_choices WHERE problem_id = ?",
            )
            .bind(problem_id)
            .fetch_optional(&state.db)
            .await?
            {
                detail.allow_multiple = Some(r.get::<i64, _>("allow_multiple") != 0);
            }
            let choice_rows = sqlx::query(
                "SELECT id, order_no, content, is_correct FROM problem_choices \
                 WHERE problem_id = ? ORDER BY order_no",
            )
            .bind(problem_id)
            .fetch_all(&state.db)
            .await?;
            detail.choices = Some(
                choice_rows
                    .iter()
                    .map(|r| Choice {
                        id: r.get("id"),
                        order_no: r.get("order_no"),
                        content: r.get("content"),
                        is_correct: r.get::<i64, _>("is_correct") != 0,
                    })
                    .collect(),
            );
        }
        "code_submit" => {
            if let Some(r) = sqlx::query(
                "SELECT input_format, output_format, constraints, \
                        time_limit_ms, memory_limit_mb, show_io_on_fail \
                 FROM problem_code_submits WHERE problem_id = ?",
            )
            .bind(problem_id)
            .fetch_optional(&state.db)
            .await?
            {
                detail.input_format = Some(r.get("input_format"));
                detail.output_format = Some(r.get("output_format"));
                detail.constraints = Some(r.get("constraints"));
                detail.time_limit_ms = Some(r.get("time_limit_ms"));
                detail.memory_limit_mb = Some(r.get("memory_limit_mb"));
                detail.show_io_on_fail = Some(r.get::<i64, _>("show_io_on_fail") != 0);
            }
            let tc_rows = sqlx::query(
                "SELECT id, number, is_sample, explanation FROM problem_test_cases \
                 WHERE problem_id = ? ORDER BY number",
            )
            .bind(problem_id)
            .fetch_all(&state.db)
            .await?;

            let mut tcs = Vec::new();
            for tc in &tc_rows {
                let number: i64 = tc.get("number");
                let dir = problem_dir(&state.data_dir, problem_id);
                let input = read_tc_content(&dir.join(format!("{number}.in"))).await;
                let expected_output = read_tc_content(&dir.join(format!("{number}.out"))).await;
                tcs.push(TestCaseDetail {
                    id: tc.get("id"),
                    number,
                    input,
                    expected_output,
                    is_sample: tc.get::<i64, _>("is_sample") != 0,
                    explanation: tc.get("explanation"),
                });
            }
            detail.test_cases = Some(tcs);
        }
        _ => {}
    }

    Ok(Json(detail))
}

/// POST /api/problems
pub async fn create_problem(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ProblemBody>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    let teacher_id = session.teacher_id.ok_or(ApiError::Forbidden)?;
    validate_body(&body)?;

    let type_id = type_id_from_slug(&body.type_slug, &state.db).await?;
    let problem_uuid = uuid::Uuid::new_v4().to_string();
    let mut tx = state.db.begin().await?;

    let problem_id: i64 = sqlx::query(
        "INSERT INTO problems (uuid, type_id, created_by, subject_id, title, description, comment, is_draft) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&problem_uuid)
    .bind(type_id)
    .bind(teacher_id)
    .bind(body.subject_id)
    .bind(body.title.trim())
    .bind(&body.description)
    .bind(&body.comment)
    .bind(if body.is_draft { 1i64 } else { 0i64 })
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    insert_type_specific(&body, problem_id, &mut tx).await?;

    // TC 파일을 커밋 전에 먼저 저장 — 실패 시 tx rollback으로 DB 불일치 없음
    if body.type_slug == "code_submit" {
        if let Some(tcs) = &body.test_cases {
            write_tc_files(&state.data_dir, problem_id, tcs).await?;
        }
    }

    tx.commit().await?;

    Ok(Json(json!({ "id": problem_id })))
}

/// PUT /api/problems/:id
pub async fn update_problem(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(problem_id): Path<i64>,
    Json(body): Json<ProblemBody>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    session.teacher_id.ok_or(ApiError::Forbidden)?;
    validate_body(&body)?;

    let mut tx = state.db.begin().await?;

    let existing_type: Option<String> = sqlx::query_scalar(
        "SELECT pt.slug FROM problems p \
         JOIN problem_types pt ON pt.id = p.type_id WHERE p.id = ?",
    )
    .bind(problem_id)
    .fetch_optional(&mut *tx)
    .await?;

    let existing_type = existing_type.ok_or(ApiError::NotFound)?;
    if existing_type != body.type_slug {
        return Err(ApiError::BadRequest("ERR_PROBLEM_TYPE_MISMATCH".into()));
    }

    sqlx::query(
        "UPDATE problems SET title = ?, description = ?, comment = ?, \
         is_draft = ?, subject_id = ? WHERE id = ?",
    )
    .bind(body.title.trim())
    .bind(&body.description)
    .bind(&body.comment)
    .bind(if body.is_draft { 1i64 } else { 0i64 })
    .bind(body.subject_id)
    .bind(problem_id)
    .execute(&mut *tx)
    .await?;

    update_type_specific(&body, problem_id, &mut tx).await?;

    // 코딩형: 커밋 전에 새 TC 파일을 임시 경로에 먼저 저장
    let pending = if body.type_slug == "code_submit" {
        let pd = state.data_dir.join("problems").join(format!("{problem_id}_pending"));
        if pd.exists() {
            tokio::fs::remove_dir_all(&pd).await.ok();
        }
        if let Some(tcs) = &body.test_cases {
            write_tc_files_to_dir(&pd, tcs).await?;
        }
        Some(pd)
    } else {
        None
    };

    if let Err(e) = tx.commit().await {
        // 커밋 실패 시 임시 디렉토리 정리
        if let Some(ref pd) = pending {
            tokio::fs::remove_dir_all(pd).await.ok();
        }
        return Err(ApiError::Database(e));
    }

    // 커밋 성공 후 기존 TC 디렉토리를 교체
    if let Some(pd) = pending {
        let actual = problem_dir(&state.data_dir, problem_id);
        if actual.exists() {
            if let Err(e) = tokio::fs::remove_dir_all(&actual).await {
                tracing::warn!("TC dir remove failed after commit: {e}");
            }
        }
        if pd.exists() {
            if let Err(e) = tokio::fs::rename(&pd, &actual).await {
                tracing::warn!("TC dir rename failed after commit: {e}");
            }
        }
    }

    Ok(Json(json!({ "ok": true })))
}

/// DELETE /api/problems/:id
pub async fn delete_problem(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(problem_id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    let session = parse_session(&headers, &state.db).await?;
    session.teacher_id.ok_or(ApiError::Forbidden)?;

    let mut tx = state.db.begin().await?;

    let type_slug: Option<String> = sqlx::query_scalar(
        "SELECT pt.slug FROM problems p \
         JOIN problem_types pt ON pt.id = p.type_id WHERE p.id = ?",
    )
    .bind(problem_id)
    .fetch_optional(&mut *tx)
    .await?;
    let type_slug = type_slug.ok_or(ApiError::NotFound)?;

    let in_lesson: Option<i64> =
        sqlx::query_scalar("SELECT 1 FROM lesson_problems WHERE problem_id = ? LIMIT 1")
            .bind(problem_id)
            .fetch_optional(&mut *tx)
            .await?;
    let in_assessment: Option<i64> =
        sqlx::query_scalar("SELECT 1 FROM assessment_problems WHERE problem_id = ? LIMIT 1")
            .bind(problem_id)
            .fetch_optional(&mut *tx)
            .await?;
    if in_lesson.is_some() || in_assessment.is_some() {
        return Err(ApiError::BadRequest("ERR_PROBLEM_IN_USE".into()));
    }

    sqlx::query("DELETE FROM problems WHERE id = ?")
        .bind(problem_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    if type_slug == "code_submit" {
        delete_tc_dir(&state.data_dir, problem_id).await?;
    }

    Ok(Json(json!({ "ok": true })))
}

// ── Type-specific DB helpers ───────────────────────────────────────────────────

async fn insert_type_specific(
    body: &ProblemBody,
    problem_id: i64,
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<(), ApiError> {
    match body.type_slug.as_str() {
        "short_answer" => {
            let answer = body.answer.as_deref().unwrap_or("").trim().to_string();
            sqlx::query(
                "INSERT INTO problem_short_answers (problem_id, answer, case_sensitive) \
                 VALUES (?, ?, ?)",
            )
            .bind(problem_id)
            .bind(&answer)
            .bind(if body.case_sensitive { 1i64 } else { 0i64 })
            .execute(&mut **tx)
            .await?;
        }
        "multiple_choice" => {
            sqlx::query(
                "INSERT INTO problem_multiple_choices (problem_id, allow_multiple) VALUES (?, ?)",
            )
            .bind(problem_id)
            .bind(if body.allow_multiple { 1i64 } else { 0i64 })
            .execute(&mut **tx)
            .await?;

            if let Some(choices) = &body.choices {
                for (i, choice) in choices.iter().enumerate() {
                    sqlx::query(
                        "INSERT INTO problem_choices (problem_id, order_no, content, is_correct) \
                         VALUES (?, ?, ?, ?)",
                    )
                    .bind(problem_id)
                    .bind(i as i64 + 1)
                    .bind(&choice.content)
                    .bind(if choice.is_correct { 1i64 } else { 0i64 })
                    .execute(&mut **tx)
                    .await?;
                }
            }
        }
        "code_submit" => {
            sqlx::query(
                "INSERT INTO problem_code_submits \
                 (problem_id, input_format, output_format, constraints, \
                  time_limit_ms, memory_limit_mb, show_io_on_fail) \
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(problem_id)
            .bind(&body.input_format)
            .bind(&body.output_format)
            .bind(&body.constraints)
            .bind(body.time_limit_ms)
            .bind(body.memory_limit_mb)
            .bind(if body.show_io_on_fail { 1i64 } else { 0i64 })
            .execute(&mut **tx)
            .await?;

            if let Some(tcs) = &body.test_cases {
                for (i, tc) in tcs.iter().enumerate() {
                    sqlx::query(
                        "INSERT INTO problem_test_cases (problem_id, number, is_sample, explanation) \
                         VALUES (?, ?, ?, ?)",
                    )
                    .bind(problem_id)
                    .bind(i as i64 + 1)
                    .bind(if tc.is_sample { 1i64 } else { 0i64 })
                    .bind(&tc.explanation)
                    .execute(&mut **tx)
                    .await?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

async fn update_type_specific(
    body: &ProblemBody,
    problem_id: i64,
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<(), ApiError> {
    match body.type_slug.as_str() {
        "short_answer" => {
            let answer = body.answer.as_deref().unwrap_or("").trim().to_string();
            sqlx::query(
                "UPDATE problem_short_answers \
                 SET answer = ?, case_sensitive = ? WHERE problem_id = ?",
            )
            .bind(&answer)
            .bind(if body.case_sensitive { 1i64 } else { 0i64 })
            .bind(problem_id)
            .execute(&mut **tx)
            .await?;
        }
        "multiple_choice" => {
            sqlx::query(
                "UPDATE problem_multiple_choices SET allow_multiple = ? WHERE problem_id = ?",
            )
            .bind(if body.allow_multiple { 1i64 } else { 0i64 })
            .bind(problem_id)
            .execute(&mut **tx)
            .await?;

            sqlx::query("DELETE FROM problem_choices WHERE problem_id = ?")
                .bind(problem_id)
                .execute(&mut **tx)
                .await?;

            if let Some(choices) = &body.choices {
                for (i, choice) in choices.iter().enumerate() {
                    sqlx::query(
                        "INSERT INTO problem_choices (problem_id, order_no, content, is_correct) \
                         VALUES (?, ?, ?, ?)",
                    )
                    .bind(problem_id)
                    .bind(i as i64 + 1)
                    .bind(&choice.content)
                    .bind(if choice.is_correct { 1i64 } else { 0i64 })
                    .execute(&mut **tx)
                    .await?;
                }
            }
        }
        "code_submit" => {
            sqlx::query(
                "UPDATE problem_code_submits \
                 SET input_format = ?, output_format = ?, constraints = ?, \
                     time_limit_ms = ?, memory_limit_mb = ?, show_io_on_fail = ? \
                 WHERE problem_id = ?",
            )
            .bind(&body.input_format)
            .bind(&body.output_format)
            .bind(&body.constraints)
            .bind(body.time_limit_ms)
            .bind(body.memory_limit_mb)
            .bind(if body.show_io_on_fail { 1i64 } else { 0i64 })
            .bind(problem_id)
            .execute(&mut **tx)
            .await?;

            sqlx::query("DELETE FROM problem_test_cases WHERE problem_id = ?")
                .bind(problem_id)
                .execute(&mut **tx)
                .await?;

            if let Some(tcs) = &body.test_cases {
                for (i, tc) in tcs.iter().enumerate() {
                    sqlx::query(
                        "INSERT INTO problem_test_cases (problem_id, number, is_sample, explanation) \
                         VALUES (?, ?, ?, ?)",
                    )
                    .bind(problem_id)
                    .bind(i as i64 + 1)
                    .bind(if tc.is_sample { 1i64 } else { 0i64 })
                    .bind(&tc.explanation)
                    .execute(&mut **tx)
                    .await?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
