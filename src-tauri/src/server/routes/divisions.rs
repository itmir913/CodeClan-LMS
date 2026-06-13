use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, server::state::AppState};
use super::auth::parse_teacher_session;

// ─── 응답 타입 ─────────────────────────────────────────────

#[derive(Serialize)]
pub struct DivisionRow {
    pub id: i64,
    pub name: String,
    pub student_count: i64,
    pub teacher_count: i64,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct TeacherBrief {
    pub id: i64,
    pub name: String,
    pub username: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct StudentRow {
    pub id: i64,
    pub student_number: String,
    pub name: String,
    pub division_id: i64,
    pub password_reset_required: bool,
    pub created_at: String,
}

// ─── 분반 목록 ─────────────────────────────────────────────

pub async fn get_divisions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<DivisionRow>>, ApiError> {
    let (teacher_id, _, role) = parse_teacher_session(&state.db, &headers).await?;

    let rows = if role == "admin" {
        sqlx::query_as::<_, (i64, String, i64, i64, String)>(
            r#"SELECT d.id, d.name,
                      (SELECT COUNT(*) FROM students s WHERE s.division_id = d.id) AS student_count,
                      (SELECT COUNT(*) FROM teacher_divisions td WHERE td.division_id = d.id) AS teacher_count,
                      d.created_at
               FROM divisions d
               ORDER BY d.name"#,
        )
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, (i64, String, i64, i64, String)>(
            r#"SELECT d.id, d.name,
                      (SELECT COUNT(*) FROM students s WHERE s.division_id = d.id) AS student_count,
                      (SELECT COUNT(*) FROM teacher_divisions td WHERE td.division_id = d.id) AS teacher_count,
                      d.created_at
               FROM divisions d
               JOIN teacher_divisions td ON td.division_id = d.id AND td.teacher_id = ?
               ORDER BY d.name"#,
        )
        .bind(teacher_id)
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(
        rows.into_iter()
            .map(|(id, name, student_count, teacher_count, created_at)| DivisionRow {
                id,
                name,
                student_count,
                teacher_count,
                created_at,
            })
            .collect(),
    ))
}

// ─── 분반 생성 (admin only) ────────────────────────────────

#[derive(Deserialize)]
pub struct CreateDivisionInput {
    pub name: String,
}

pub async fn create_division(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreateDivisionInput>,
) -> Result<Json<DivisionRow>, ApiError> {
    let (teacher_id, _, role) = parse_teacher_session(&state.db, &headers).await?;
    if role != "admin" {
        return Err(ApiError::Forbidden);
    }

    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::BadRequest("분반 이름을 입력하세요".into()));
    }

    let id = sqlx::query("INSERT INTO divisions (name) VALUES (?)")
        .bind(&name)
        .execute(&state.db)
        .await?
        .last_insert_rowid();

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'division_create', 'division', ?, ?)",
    )
    .bind(teacher_id)
    .bind(id)
    .bind(format!("분반 생성: {name}"))
    .execute(&state.db)
    .await;

    Ok(Json(DivisionRow {
        id,
        name,
        student_count: 0,
        teacher_count: 0,
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }))
}

// ─── 분반 수정 (admin only) ────────────────────────────────

#[derive(Deserialize)]
pub struct UpdateDivisionInput {
    pub name: String,
}

pub async fn update_division(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<UpdateDivisionInput>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (teacher_id, _, role) = parse_teacher_session(&state.db, &headers).await?;
    if role != "admin" {
        return Err(ApiError::Forbidden);
    }

    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::BadRequest("분반 이름을 입력하세요".into()));
    }

    let rows = sqlx::query("UPDATE divisions SET name = ? WHERE id = ?")
        .bind(&name)
        .bind(id)
        .execute(&state.db)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(ApiError::NotFound);
    }

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'division_update', 'division', ?, ?)",
    )
    .bind(teacher_id)
    .bind(id)
    .bind(format!("분반 이름 변경: {name}"))
    .execute(&state.db)
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── 분반 삭제 (admin only) ────────────────────────────────

pub async fn delete_division(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (teacher_id, _, role) = parse_teacher_session(&state.db, &headers).await?;
    if role != "admin" {
        return Err(ApiError::Forbidden);
    }

    let student_count: i64 =
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM students WHERE division_id = ?")
            .bind(id)
            .fetch_one(&state.db)
            .await?
            .0;

    if student_count > 0 {
        return Err(ApiError::BadRequest(
            "학생이 있는 분반은 삭제할 수 없습니다. 먼저 학생을 모두 이동하거나 삭제하세요.".into(),
        ));
    }

    let rows = sqlx::query("DELETE FROM divisions WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(ApiError::NotFound);
    }

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'division_delete', 'division', ?, ?)",
    )
    .bind(teacher_id)
    .bind(id)
    .bind("분반 삭제")
    .execute(&state.db)
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── 분반 담당 교사 조회 ───────────────────────────────────

pub async fn get_division_teachers(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<Vec<TeacherBrief>>, ApiError> {
    let (_, _, role) = parse_teacher_session(&state.db, &headers).await?;
    if role != "admin" {
        return Err(ApiError::Forbidden);
    }

    let rows = sqlx::query_as::<_, (i64, String, String, String)>(
        r#"SELECT t.id, t.name, t.username, t.role
           FROM teachers t
           JOIN teacher_divisions td ON td.teacher_id = t.id
           WHERE td.division_id = ?
           ORDER BY t.name"#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, name, username, role)| TeacherBrief { id, name, username, role })
            .collect(),
    ))
}

// ─── 분반 담당 교사 설정 (admin only) ─────────────────────

#[derive(Deserialize)]
pub struct SetTeachersInput {
    pub teacher_ids: Vec<i64>,
}

pub async fn set_division_teachers(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<SetTeachersInput>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (actor_id, _, role) = parse_teacher_session(&state.db, &headers).await?;
    if role != "admin" {
        return Err(ApiError::Forbidden);
    }

    let mut tx = state.db.begin().await?;

    sqlx::query("DELETE FROM teacher_divisions WHERE division_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    for tid in &input.teacher_ids {
        sqlx::query("INSERT INTO teacher_divisions (teacher_id, division_id) VALUES (?, ?)")
            .bind(tid)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'division_teachers_update', 'division', ?, ?)",
    )
    .bind(actor_id)
    .bind(id)
    .bind(format!("담당 교사 {}명 설정", input.teacher_ids.len()))
    .execute(&state.db)
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── 분반 학생 목록 ────────────────────────────────────────

pub async fn get_students(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(division_id): Path<i64>,
) -> Result<Json<Vec<StudentRow>>, ApiError> {
    let (teacher_id, _, role) = parse_teacher_session(&state.db, &headers).await?;

    // 일반 교사는 담당 분반만 조회 가능
    if role != "admin" {
        let assigned: Option<(i64,)> = sqlx::query_as::<_, (i64,)>(
            "SELECT division_id FROM teacher_divisions WHERE teacher_id = ? AND division_id = ?",
        )
        .bind(teacher_id)
        .bind(division_id)
        .fetch_optional(&state.db)
        .await?;

        if assigned.is_none() {
            return Err(ApiError::Forbidden);
        }
    }

    let rows = sqlx::query_as::<_, (i64, String, String, i64, i64, String)>(
        r#"SELECT id, student_number, name, division_id, password_reset_required, created_at
           FROM students WHERE division_id = ?
           ORDER BY student_number"#,
    )
    .bind(division_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, student_number, name, division_id, prr, created_at)| StudentRow {
                id,
                student_number,
                name,
                division_id,
                password_reset_required: prr != 0,
                created_at,
            })
            .collect(),
    ))
}

// ─── 학생 단건 추가 ────────────────────────────────────────

#[derive(Deserialize)]
pub struct AddStudentInput {
    pub student_number: String,
    pub name: String,
    pub password: String,
}

pub async fn add_student(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(division_id): Path<i64>,
    Json(input): Json<AddStudentInput>,
) -> Result<Json<StudentRow>, ApiError> {
    let (teacher_id, _, role) = parse_teacher_session(&state.db, &headers).await?;
    if role != "admin" {
        return Err(ApiError::Forbidden);
    }

    let student_number = input.student_number.trim().to_string();
    let name = input.name.trim().to_string();

    if student_number.is_empty() || name.is_empty() || input.password.is_empty() {
        return Err(ApiError::BadRequest("학번, 이름, 초기 비밀번호를 모두 입력하세요".into()));
    }

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(input.password.as_bytes(), &salt)
        .map_err(|e| ApiError::InternalError(e.to_string()))?
        .to_string();

    let id = sqlx::query(
        r#"INSERT INTO students (division_id, student_number, name, password_hash, password_reset_required)
           VALUES (?, ?, ?, ?, 1)"#,
    )
    .bind(division_id)
    .bind(&student_number)
    .bind(&name)
    .bind(&hash)
    .execute(&state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            ApiError::BadRequest(format!("학번 {student_number}은(는) 이미 등록되어 있습니다"))
        } else {
            ApiError::from(e)
        }
    })?
    .last_insert_rowid();

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'student_add', 'student', ?, ?)",
    )
    .bind(teacher_id)
    .bind(id)
    .bind(format!("학생 추가: {name} ({student_number})"))
    .execute(&state.db)
    .await;

    Ok(Json(StudentRow {
        id,
        student_number,
        name,
        division_id,
        password_reset_required: true,
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }))
}

// ─── 학생 일괄 등록 ────────────────────────────────────────

#[derive(Deserialize)]
pub struct BulkStudentItem {
    pub student_number: String,
    pub name: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct BulkImportResult {
    pub inserted: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

pub async fn bulk_import_students(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(division_id): Path<i64>,
    Json(items): Json<Vec<BulkStudentItem>>,
) -> Result<Json<BulkImportResult>, ApiError> {
    let (teacher_id, _, role) = parse_teacher_session(&state.db, &headers).await?;
    if role != "admin" {
        return Err(ApiError::Forbidden);
    }

    if items.is_empty() {
        return Err(ApiError::BadRequest("등록할 학생 목록이 비어 있습니다".into()));
    }

    let argon2 = Argon2::default();
    let mut inserted = 0usize;
    let mut skipped = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for item in &items {
        let student_number = item.student_number.trim().to_string();
        let name = item.name.trim().to_string();

        if student_number.is_empty() || name.is_empty() || item.password.is_empty() {
            errors.push(format!("{}: 학번/이름/비밀번호 중 빈 값이 있습니다", student_number));
            continue;
        }

        let salt = SaltString::generate(&mut OsRng);
        let hash = match argon2.hash_password(item.password.as_bytes(), &salt) {
            Ok(h) => h.to_string(),
            Err(e) => {
                errors.push(format!("{student_number}: 비밀번호 해시 실패 ({e})"));
                continue;
            }
        };

        let result = sqlx::query(
            r#"INSERT OR IGNORE INTO students (division_id, student_number, name, password_hash, password_reset_required)
               VALUES (?, ?, ?, ?, 1)"#,
        )
        .bind(division_id)
        .bind(&student_number)
        .bind(&name)
        .bind(&hash)
        .execute(&state.db)
        .await;

        match result {
            Ok(r) if r.rows_affected() > 0 => inserted += 1,
            Ok(_) => skipped += 1,
            Err(e) => errors.push(format!("{student_number}: DB 오류 ({e})")),
        }
    }

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'student_bulk_import', 'division', ?, ?)",
    )
    .bind(teacher_id)
    .bind(division_id)
    .bind(format!("{inserted}명 일괄 등록 (건너뜀 {skipped}, 오류 {})", errors.len()))
    .execute(&state.db)
    .await;

    Ok(Json(BulkImportResult { inserted, skipped, errors }))
}

// ─── 학생 삭제 (admin only) ────────────────────────────────

pub async fn delete_student(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(student_id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (teacher_id, _, role) = parse_teacher_session(&state.db, &headers).await?;
    if role != "admin" {
        return Err(ApiError::Forbidden);
    }

    let rows = sqlx::query("DELETE FROM students WHERE id = ?")
        .bind(student_id)
        .execute(&state.db)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(ApiError::NotFound);
    }

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'student_delete', 'student', ?, ?)",
    )
    .bind(teacher_id)
    .bind(student_id)
    .bind("학생 삭제")
    .execute(&state.db)
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── 학생 비밀번호 초기화 (담당 교사 or admin) ────────────

#[derive(Deserialize)]
pub struct ResetPasswordInput {
    pub new_password: String,
}

pub async fn reset_student_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(student_id): Path<i64>,
    Json(input): Json<ResetPasswordInput>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (teacher_id, _, role) = parse_teacher_session(&state.db, &headers).await?;

    // 담당 분반 여부 확인 (admin은 패스)
    if role != "admin" {
        let ok: Option<(i64,)> = sqlx::query_as::<_, (i64,)>(
            r#"SELECT s.id FROM students s
               JOIN teacher_divisions td ON td.division_id = s.division_id
               WHERE s.id = ? AND td.teacher_id = ?"#,
        )
        .bind(student_id)
        .bind(teacher_id)
        .fetch_optional(&state.db)
        .await?;

        if ok.is_none() {
            return Err(ApiError::Forbidden);
        }
    }

    if input.new_password.is_empty() {
        return Err(ApiError::BadRequest("새 비밀번호를 입력하세요".into()));
    }

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(input.new_password.as_bytes(), &salt)
        .map_err(|e| ApiError::InternalError(e.to_string()))?
        .to_string();

    sqlx::query(
        "UPDATE students SET password_hash = ?, password_reset_required = 1 WHERE id = ?",
    )
    .bind(&hash)
    .bind(student_id)
    .execute(&state.db)
    .await?;

    let _ = sqlx::query(
        "INSERT INTO audit_logs (actor_teacher_id, action_type, target_type, target_id, detail) VALUES (?, 'student_password_reset', 'student', ?, ?)",
    )
    .bind(teacher_id)
    .bind(student_id)
    .bind("학생 비밀번호 초기화")
    .execute(&state.db)
    .await;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── 분반 연결 수행평가 목록 ───────────────────────────────

#[derive(Serialize)]
pub struct DivisionAssessmentRow {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub problem_count: i64,
}

pub async fn get_division_assessments(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(division_id): Path<i64>,
) -> Result<Json<Vec<DivisionAssessmentRow>>, ApiError> {
    parse_teacher_session(&state.db, &headers).await?;

    let rows = sqlx::query_as::<_, (i64, String, String, i64)>(
        r#"SELECT a.id, a.title, a.description,
                  (SELECT COUNT(*) FROM assessment_problems ap WHERE ap.assessment_id = a.id) AS problem_count
           FROM assessments a
           JOIN assessment_divisions ad ON ad.assessment_id = a.id
           WHERE ad.division_id = ?
           ORDER BY a.created_at DESC"#,
    )
    .bind(division_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|(id, title, description, problem_count)| DivisionAssessmentRow {
                id, title, description, problem_count,
            })
            .collect(),
    ))
}
