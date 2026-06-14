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

/// GET /api/setup/status
pub async fn get_status(State(state): State<AppState>) -> Result<Json<SetupStatus>, ApiError> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM app_configs WHERE key = 'setup_complete'",
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(SetupStatus {
        needs_setup: count == 0,
    }))
}

#[derive(Deserialize)]
pub struct SetupRequest {
    pub school_name: String,
    pub admin_name: String,
    pub admin_username: String,
    pub admin_password: String,
}

/// POST /api/setup/complete
pub async fn complete(
    State(state): State<AppState>,
    Json(body): Json<SetupRequest>,
) -> Result<Json<Value>, ApiError> {
    let already_set = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM app_configs WHERE key = 'setup_complete'",
    )
    .fetch_one(&state.db)
    .await?;
    if already_set > 0 {
        return Err(ApiError::BadRequest("이미 초기 설정이 완료되었습니다".into()));
    }

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

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(body.admin_password.as_bytes(), &salt)
        .map_err(|e| ApiError::Internal(format!("Password hash error: {e}")))?
        .to_string();

    let mut tx = state.db.begin().await?;

    sqlx::query(
        "INSERT INTO app_configs (key, value) VALUES ('school_name', ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
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

    sqlx::query(
        "INSERT INTO app_configs (key, value) VALUES ('setup_complete', '1')
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    tracing::info!("Initial setup complete: school={school_name}, admin={admin_username}");
    Ok(Json(json!({ "ok": true })))
}
