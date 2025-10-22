use crate::DbError;
use crate::models::{Game, GroupGameMap};
use itertools::{Either, Itertools};
use sqlx::SqlitePool;
use wwc_core::game::GameId;
use wwc_core::group::{
    GroupId,
    game::{PlayedGroupGame, UnplayedGroupGame},
};

/// Get all group games from the database
pub async fn get_group_games(
    pool: &SqlitePool,
) -> Result<(Vec<PlayedGroupGame>, Vec<UnplayedGroupGame>), DbError> {
    let group_games = sqlx::query_as::<_, Game>("SELECT * FROM games WHERE type_ = ?")
        .bind("group")
        .fetch_all(pool)
        .await
        .map_err(DbError::Sqlx)?;

    type FetchedPlayedGroupGame = Vec<Result<PlayedGroupGame, DbError>>;
    type FetchedUnplayedGroupGame = Vec<Result<UnplayedGroupGame, DbError>>;

    let (played_games, unplayed_games): (FetchedPlayedGroupGame, FetchedUnplayedGroupGame) =
        group_games.into_iter().partition_map(|game| {
            if game.played {
                Either::Left(PlayedGroupGame::try_from(game))
            } else {
                Either::Right(UnplayedGroupGame::try_from(game))
            }
        });

    let played_games = played_games
        .into_iter()
        .collect::<Result<Vec<PlayedGroupGame>, DbError>>()?;
    let unplayed_games = unplayed_games
        .into_iter()
        .collect::<Result<Vec<UnplayedGroupGame>, DbError>>()?;

    Ok((played_games, unplayed_games))
}

/// Get group-to-game mappings from the database
pub async fn get_group_game_maps(pool: &SqlitePool) -> Result<Vec<(GameId, GroupId)>, DbError> {
    let db_maps = sqlx::query_as::<_, GroupGameMap>("SELECT * FROM group_game_map")
        .fetch_all(pool)
        .await
        .map_err(DbError::Sqlx)?;

    Ok(db_maps
        .into_iter()
        .map(|map_| {
            (
                GameId::from(u32::try_from(map_.id).unwrap()),
                GroupId::try_from(map_.group_id_.chars().next().unwrap()).unwrap(),
            )
        })
        .collect())
}

/// Insert group game mappings into the database
pub async fn insert_group_game_mappings(
    pool: &SqlitePool,
    group_mappings: &[(GroupId, GameId)],
) -> Result<(), DbError> {
    for (group_id, game_id) in group_mappings {
        let group_str = String::from(char::from(*group_id));
        sqlx::query("INSERT INTO group_game_map (id, group_id_) VALUES (?, ?)")
            .bind(i32::try_from(u32::from(*game_id)).unwrap())
            .bind(group_str)
            .execute(pool)
            .await
            .map_err(DbError::Sqlx)?;
    }

    Ok(())
}

/// Clear all group game mappings from the database
pub async fn clear_group_game_maps(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::query("DELETE FROM group_game_map")
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    Ok(())
}
