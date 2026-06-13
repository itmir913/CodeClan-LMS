use axum::{extract::State, Json};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{error::ApiError, server::state::AppState};

#[derive(Serialize)]
pub struct SetupStatus {
    pub needs_setup: bool,
}

/// GET /api/setup/status — teachers 테이블이 비어있으면 needs_setup=true
pub async fn get_status(State(state): State<AppState>) -> Result<Json<SetupStatus>, ApiError> {
    let row = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM teachers")
        .fetch_one(&state.db)
        .await?;

    Ok(Json(SetupStatus {
        needs_setup: row == 0,
    }))
}

#[derive(Deserialize)]
pub struct SetupRequest {
    pub school_name: String,
    pub admin_name: String,
    pub admin_username: String,
    pub admin_password: String,
}

/// POST /api/setup/complete — 학교 이름 저장 + 관리자 계정 생성
pub async fn complete(
    State(state): State<AppState>,
    Json(body): Json<SetupRequest>,
) -> Result<Json<Value>, ApiError> {
    // 이미 설정된 경우 차단
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM teachers")
        .fetch_one(&state.db)
        .await?;
    if count > 0 {
        return Err(ApiError::BadRequest("이미 초기 설정이 완료되었습니다".into()));
    }

    // 입력 검증
    let school_name = body.school_name.trim().to_string();
    let admin_name = body.admin_name.trim().to_string();
    let admin_username = body.admin_username.trim().to_string();

    if school_name.is_empty() {
        return Err(ApiError::BadRequest("학교 이름을 입력해주세요".into()));
    }
    if admin_name.is_empty() {
        return Err(ApiError::BadRequest("관리자 이름을 입력해주세요".into()));
    }
    if admin_username.is_empty() {
        return Err(ApiError::BadRequest("아이디를 입력해주세요".into()));
    }
    if body.admin_password.len() < 8 {
        return Err(ApiError::BadRequest("비밀번호는 8자 이상이어야 합니다".into()));
    }

    // 비밀번호 해시 (argon2)
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(body.admin_password.as_bytes(), &salt)
        .map_err(|e| ApiError::Internal(format!("Password hash error: {e}")))?
        .to_string();

    // 트랜잭션으로 settings + teacher 동시 저장
    let mut tx = state.db.begin().await?;

    sqlx::query(
        "INSERT INTO settings (id, school_name) VALUES (1, ?)
         ON CONFLICT(id) DO UPDATE SET school_name = excluded.school_name, updated_at = datetime('now')",
    )
    .bind(&school_name)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO teachers (username, password_hash, name, role) VALUES (?, ?, ?, 'admin')",
    )
    .bind(&admin_username)
    .bind(&password_hash)
    .bind(&admin_name)
    .execute(&mut *tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.message().contains("UNIQUE") => {
            ApiError::BadRequest("이미 사용 중인 아이디입니다".into())
        }
        other => ApiError::Database(other),
    })?;

    tx.commit().await?;

    tracing::info!("Initial setup complete: school={school_name}, admin={admin_username}");
    Ok(Json(json!({ "ok": true })))
}
