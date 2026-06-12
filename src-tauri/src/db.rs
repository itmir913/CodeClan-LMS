use anyhow::Context;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;

pub async fn init(db_path: &Path) -> anyhow::Result<SqlitePool> {
    if let Some(parent) = db_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .context("Failed to create data directory")?;
    }

    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(1) // SQLite 단일 writer 큐
        .connect(&db_url)
        .await
        .context("Failed to connect to SQLite")?;

    // WAL 모드 활성화
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await
        .context("Failed to enable WAL")?;

    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await
        .context("Failed to enable foreign keys")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run migrations")?;

    tracing::info!("Database initialized: {}", db_path.display());
    Ok(pool)
}
