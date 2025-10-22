//! Team operations

use crate::{DbError, models::Team};
use sqlx::SqlitePool;

/// Get all teams from the database
pub async fn get_teams(pool: &SqlitePool) -> Result<Vec<wwc_core::Team>, DbError> {
    let db_teams = sqlx::query_as::<_, Team>("SELECT * FROM teams")
        .fetch_all(pool)
        .await
        .map_err(DbError::Sqlx)?;

    Ok(db_teams.into_iter().map(|team| team.into()).collect())
}

/// Insert teams into the database
pub async fn insert_teams(pool: &SqlitePool, teams: &[wwc_core::Team]) -> Result<(), DbError> {
    for team in teams {
        let name: &str = team.name.as_ref();
        let fifa_code: &str = team.fifa_code.as_ref();
        sqlx::query("INSERT INTO teams (id, name, fifa_code, rank_) VALUES (?, ?, ?, ?)")
            .bind(i32::try_from(u32::from(team.id)).unwrap())
            .bind(name)
            .bind(fifa_code)
            .bind(i32::try_from(u32::from(team.rank)).unwrap())
            .execute(pool)
            .await
            .map_err(DbError::Sqlx)?;
    }

    Ok(())
}

/// Clear all teams from the database
pub async fn clear_teams(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::query("DELETE FROM teams")
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    Ok(())
}
