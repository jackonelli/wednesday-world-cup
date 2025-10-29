use crate::DbError;
use crate::models::{Bot, User};
use sqlx::SqlitePool;

/// Create a new user (user.id must match player.id)
pub async fn create_user(
    pool: &SqlitePool,
    player_id: i32,
    username: &str,
    password_hash: &str,
    display_name: &str,
) -> Result<(), DbError> {
    sqlx::query(
        "INSERT INTO users (id, username, password_hash, display_name) VALUES (?, ?, ?, ?)",
    )
    .bind(player_id)
    .bind(username)
    .bind(password_hash)
    .bind(display_name)
    .execute(pool)
    .await
    .map_err(DbError::Sqlx)?;

    Ok(())
}

/// Get user by username
pub async fn get_user_by_username(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<User>, DbError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(DbError::Sqlx)?;

    Ok(user)
}

/// Get user by ID
pub async fn get_user_by_id(pool: &SqlitePool, user_id: i32) -> Result<Option<User>, DbError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(DbError::Sqlx)?;

    Ok(user)
}

/// List all users
pub async fn list_users(pool: &SqlitePool) -> Result<Vec<User>, DbError> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY username")
        .fetch_all(pool)
        .await
        .map_err(DbError::Sqlx)?;

    Ok(users)
}

/// Delete a user by username (bots will be cascade deleted)
pub async fn delete_user(pool: &SqlitePool, username: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM users WHERE username = ?")
        .bind(username)
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;

    Ok(())
}

/// Create a bot for a user
pub async fn create_bot(
    pool: &SqlitePool,
    user_id: i32,
    bot_name: &str,
    bot_display_name: &str,
) -> Result<i64, DbError> {
    let result =
        sqlx::query("INSERT INTO bots (user_id, bot_name, bot_display_name) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(bot_name)
            .bind(bot_display_name)
            .execute(pool)
            .await
            .map_err(DbError::Sqlx)?;

    Ok(result.last_insert_rowid())
}

/// Get bot by user_id and bot_name
pub async fn get_bot(
    pool: &SqlitePool,
    user_id: i32,
    bot_name: &str,
) -> Result<Option<Bot>, DbError> {
    let bot = sqlx::query_as::<_, Bot>("SELECT * FROM bots WHERE user_id = ? AND bot_name = ?")
        .bind(user_id)
        .bind(bot_name)
        .fetch_optional(pool)
        .await
        .map_err(DbError::Sqlx)?;

    Ok(bot)
}

/// List all bots for a user
pub async fn list_bots_for_user(pool: &SqlitePool, user_id: i32) -> Result<Vec<Bot>, DbError> {
    let bots = sqlx::query_as::<_, Bot>("SELECT * FROM bots WHERE user_id = ? ORDER BY created_at")
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(DbError::Sqlx)?;

    Ok(bots)
}

/// Delete a bot by user_id and bot_name
pub async fn delete_bot(pool: &SqlitePool, user_id: i32, bot_name: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM bots WHERE user_id = ? AND bot_name = ?")
        .bind(user_id)
        .bind(bot_name)
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;

    Ok(())
}

/// Get all display names (user display names + bot display names)
/// Returns Vec<(player_id, bot_name, display_name)>
pub async fn get_all_display_names(
    pool: &SqlitePool,
) -> Result<Vec<(i32, Option<String>, String)>, DbError> {
    let results = sqlx::query_as::<_, (i32, Option<String>, String)>(
        r#"
        SELECT u.id as player_id, NULL as bot_name, u.display_name
        FROM users u
        UNION ALL
        SELECT b.user_id as player_id, b.bot_name, b.bot_display_name as display_name
        FROM bots b
        ORDER BY player_id, bot_name
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(DbError::Sqlx)?;

    Ok(results)
}
