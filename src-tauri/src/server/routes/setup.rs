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
    pub locale: Option<String>,
}

/// GET /api/setup/status
pub async fn get_status(State(state): State<AppState>) -> Result<Json<SetupStatus>, ApiError> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM app_configs WHERE key = 'setup_complete'",
    )
    .fetch_one(&state.db)
    .await?;

    let locale = sqlx::query_scalar::<_, String>(
        "SELECT value FROM app_configs WHERE key = 'locale'",
    )
    .fetch_optional(&state.db)
    .await?;

    Ok(Json(SetupStatus {
        needs_setup: count == 0,
        locale,
    }))
}

#[derive(Deserialize)]
pub struct SetupRequest {
    pub school_name: String,
    pub admin_name: String,
    pub admin_username: String,
    pub admin_password: String,
    pub locale: String,
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
        return Err(ApiError::BadRequest("ERR_SETUP_ALREADY_COMPLETE".into()));
    }

    let school_name = body.school_name.trim().to_string();
    let admin_name = body.admin_name.trim().to_string();
    let admin_username = body.admin_username.trim().to_string();

    // 형식이 올바르지 않은 locale은 기본값 en으로 대체
    let raw_locale = body.locale.trim().to_string();
    let locale = if raw_locale.is_empty()
        || raw_locale.len() > 10
        || !raw_locale.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        "en".to_string()
    } else {
        raw_locale
    };

    if school_name.is_empty() {
        return Err(ApiError::BadRequest("ERR_SCHOOL_NAME_REQUIRED".into()));
    }
    if admin_name.is_empty() {
        return Err(ApiError::BadRequest("ERR_ADMIN_NAME_REQUIRED".into()));
    }
    if admin_username.is_empty() {
        return Err(ApiError::BadRequest("ERR_USERNAME_REQUIRED".into()));
    }
    if body.admin_password.len() < 8 {
        return Err(ApiError::BadRequest("ERR_PASSWORD_TOO_SHORT".into()));
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(body.admin_password.as_bytes(), &salt)
        .map_err(|e| ApiError::Internal(format!("argon2: {e}")))?
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
        "INSERT INTO app_configs (key, value) VALUES ('locale', ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(&locale)
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
            ApiError::BadRequest("ERR_USERNAME_TAKEN".into())
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
