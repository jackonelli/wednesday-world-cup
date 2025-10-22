//! Database connection pool and migrations

use crate::DbError;
use dotenv::dotenv;
use sqlx::SqlitePool;
use std::env;

/// Create a SQLite connection pool and run migrations
pub async fn create_pool() -> Result<SqlitePool, DbError> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").map_err(|_| DbError::DbUrlMissing)?;

    // Create the database file if it doesn't exist
    if !database_url.contains(":memory:") {
        let db_path = database_url
            .strip_prefix("sqlite:")
            .unwrap_or(&database_url);
        if let Some(parent) = std::path::Path::new(db_path).parent() {
            std::fs::create_dir_all(parent).ok();
        }
    }

    let pool = SqlitePool::connect(&database_url)
        .await
        .map_err(DbError::Sqlx)?;

    // Run migrations (idempotent due to IF NOT EXISTS)
    sqlx::query(include_str!("../sqlx_migrations/001_create_tables.sql"))
        .execute(&pool)
        .await
        .map_err(DbError::Sqlx)?;

    sqlx::query(include_str!("../sqlx_migrations/002_playoff_tables.sql"))
        .execute(&pool)
        .await
        .map_err(DbError::Sqlx)?;

    Ok(pool)
}
