use crate::DbError;
use crate::models::{Player, Pred};
use sqlx::SqlitePool;
use wwc_core::player::{PlayerId, PlayerPredictions, Prediction};

/// Register a new player in the database
pub async fn register_player(pool: &SqlitePool, name: &str) -> Result<(), DbError> {
    let existing = sqlx::query_as::<_, Player>("SELECT * FROM players WHERE name = ?")
        .bind(name)
        .fetch_optional(pool)
        .await
        .map_err(DbError::Sqlx)?;

    if existing.is_some() {
        return Err(DbError::Generic(format!(
            "Player with name: '{}' already in db",
            name
        )));
    }

    sqlx::query("INSERT INTO players (name) VALUES (?)")
        .bind(name)
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;

    Ok(())
}

/// Get all predictions for a specific player and optional bot
pub async fn get_preds(
    pool: &SqlitePool,
    player_id: PlayerId,
    bot_name: Option<&str>,
) -> Result<Vec<Prediction>, DbError> {
    let player_id = i32::from(player_id);

    let db_preds = if let Some(bot) = bot_name {
        // If bot_name is specified, filter by it
        sqlx::query_as::<_, Pred>("SELECT * FROM preds WHERE player_id = ? AND bot_name = ?")
            .bind(player_id)
            .bind(bot)
            .fetch_all(pool)
            .await
            .map_err(DbError::Sqlx)?
    } else {
        // If no bot_name specified, get all predictions for this player (backwards compatible)
        sqlx::query_as::<_, Pred>("SELECT * FROM preds WHERE player_id = ?")
            .bind(player_id)
            .fetch_all(pool)
            .await
            .map_err(DbError::Sqlx)?
    };

    Ok(db_preds.into_iter().map(Prediction::from).collect())
}

/// Get all players from the database
pub async fn get_players(pool: &SqlitePool) -> Result<Vec<Player>, DbError> {
    let players = sqlx::query_as::<_, Player>("SELECT * FROM players")
        .fetch_all(pool)
        .await
        .map_err(DbError::Sqlx)?;

    Ok(players)
}

/// Insert predictions for a player (replaces existing predictions for that player+bot combination)
pub async fn insert_preds(
    pool: &SqlitePool,
    preds: &PlayerPredictions,
    bot_name: Option<&str>,
) -> Result<(), DbError> {
    let player_id = i32::from(preds.id);

    // Delete existing predictions for this player+bot combination
    if let Some(bot) = bot_name {
        sqlx::query("DELETE FROM preds WHERE player_id = ? AND bot_name = ?")
            .bind(player_id)
            .bind(bot)
            .execute(pool)
            .await
            .map_err(DbError::Sqlx)?;
    } else {
        sqlx::query("DELETE FROM preds WHERE player_id = ? AND bot_name IS NULL")
            .bind(player_id)
            .execute(pool)
            .await
            .map_err(DbError::Sqlx)?;
    }

    // Insert new predictions
    for pred in preds.preds() {
        sqlx::query(
            "INSERT INTO preds (player_id, game_id, home_result, away_result, bot_name) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(player_id)
        .bind(i32::try_from(u32::from(pred.0)).unwrap())
        .bind(i32::try_from(u32::from(pred.1.home)).unwrap())
        .bind(i32::try_from(u32::from(pred.1.away)).unwrap())
        .bind(bot_name)
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    }

    Ok(())
}

/// Clear all predictions from the database (admin only)
pub async fn clear_preds(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::query("DELETE FROM preds")
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    Ok(())
}

/// Clear predictions for a specific player and optional bot
pub async fn clear_player_preds(
    pool: &SqlitePool,
    player_id: PlayerId,
    bot_name: Option<&str>,
) -> Result<(), DbError> {
    let player_id = i32::from(player_id);

    if let Some(bot) = bot_name {
        sqlx::query("DELETE FROM preds WHERE player_id = ? AND bot_name = ?")
            .bind(player_id)
            .bind(bot)
            .execute(pool)
            .await
            .map_err(DbError::Sqlx)?;
    } else {
        // Clear all predictions for this player (all bots + human)
        sqlx::query("DELETE FROM preds WHERE player_id = ?")
            .bind(player_id)
            .execute(pool)
            .await
            .map_err(DbError::Sqlx)?;
    }
    Ok(())
}

/// Clear all players from the database
pub async fn clear_players(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::query("DELETE FROM players")
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    Ok(())
}
