//! Shared game operations (used by both group and playoff)

use crate::{DbError, models::Game};
use sqlx::SqlitePool;
use wwc_core::group::game::{PlayedGroupGame, UnplayedGroupGame};

/// Get all games from the database
pub async fn get_games(pool: &SqlitePool) -> Result<Vec<Game>, DbError> {
    let games = sqlx::query_as::<_, Game>("SELECT * FROM games")
        .fetch_all(pool)
        .await
        .map_err(DbError::Sqlx)?;

    Ok(games)
}

/// Insert unplayed games into the database
pub async fn insert_unplayed_games(
    pool: &SqlitePool,
    games: &[UnplayedGroupGame],
) -> Result<(), DbError> {
    for game in games {
        sqlx::query(
            "INSERT INTO games (id, type_, home_team, away_team, home_result, away_result, \
             home_penalty, away_penalty, home_fair_play, away_fair_play, played) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(i32::try_from(u32::from(game.id)).unwrap())
        .bind("group")
        .bind(i32::try_from(u32::from(game.home)).unwrap())
        .bind(i32::try_from(u32::from(game.away)).unwrap())
        .bind(None::<i32>)
        .bind(None::<i32>)
        .bind(None::<i32>)
        .bind(None::<i32>)
        .bind(None::<i32>)
        .bind(None::<i32>)
        .bind(false)
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    }

    Ok(())
}

/// Insert played games into the database
pub async fn insert_played_games(
    pool: &SqlitePool,
    games: &[PlayedGroupGame],
) -> Result<(), DbError> {
    for game in games {
        sqlx::query(
            "INSERT INTO games (id, type_, home_team, away_team, home_result, away_result, \
             home_penalty, away_penalty, home_fair_play, away_fair_play, played) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(i32::try_from(u32::from(game.id)).unwrap())
        .bind("group")
        .bind(i32::try_from(u32::from(game.home)).unwrap())
        .bind(i32::try_from(u32::from(game.away)).unwrap())
        .bind(Some(i32::try_from(u32::from(game.score.home)).unwrap()))
        .bind(Some(i32::try_from(u32::from(game.score.away)).unwrap()))
        .bind(None::<i32>)
        .bind(None::<i32>)
        .bind(None::<i32>)
        .bind(None::<i32>)
        .bind(true)
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    }

    Ok(())
}

/// Clear all games from the database
pub async fn clear_games(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::query("DELETE FROM games")
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    Ok(())
}
