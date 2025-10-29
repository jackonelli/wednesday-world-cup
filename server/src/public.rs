use crate::AppError;

use axum::{Json, extract::State};
use itertools::Itertools;
use sqlx::SqlitePool;
use std::collections::{BTreeMap, HashMap};
use tracing::info;
use wwc_core::error::WwcError;
use wwc_core::game::GameId;
use wwc_core::group::{Group, GroupId, Groups, game::PlayedGroupGame, game::UnplayedGroupGame};
use wwc_core::player::PlayerId;
use wwc_core::playoff::TeamSource;
use wwc_core::team::Teams;

/// Get teams
pub(crate) async fn get_teams(State(pool): State<SqlitePool>) -> Result<Json<Teams>, AppError> {
    let teams: Teams = wwc_db::get_teams(&pool)
        .await?
        .into_iter()
        .map(|x| (x.id, x))
        .collect();

    info!("Retrieved {} teams", teams.len());
    Ok(Json(teams))
}

/// Get playoff team sources
///
/// Returns the team sources for all playoff games.
/// This is used to build the BracketStructure on the client.
pub(crate) async fn get_playoff_team_sources(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<(GameId, (TeamSource, TeamSource))>>, AppError> {
    let team_sources = wwc_db::get_playoff_team_sources(&pool).await?;

    info!("Retrieved {} playoff team sources", team_sources.len());
    Ok(Json(team_sources))
}

/// Get groups
///
/// Loads group games and a GameId: GroupId map from the db.
/// The games (played and unplayed) are then mapped to prospective groups.
/// The final groups are validated (with a fallible constructor) and collected together.
pub(crate) async fn get_groups(State(pool): State<SqlitePool>) -> Result<Json<Groups>, AppError> {
    let (played_games, unplayed_games) = wwc_db::get_group_games(&pool).await?;

    let game_group_map = wwc_db::get_group_game_maps(&pool)
        .await?
        .into_iter()
        .collect::<HashMap<GameId, GroupId>>();

    let empty_groups = game_group_map
        .iter()
        .map(|(_game_id, group_id)| group_id)
        .unique();

    let groups_played =
        played_games.into_iter().fold(
            empty_groups
                .clone()
                .map(|group_id| (*group_id, Vec::new()))
                .collect::<BTreeMap<GroupId, Vec<PlayedGroupGame>>>(),
            |mut acc, game| {
                let entry = acc
                    .entry(*game_group_map.get(&game.id).unwrap_or_else(|| {
                        panic!("game group map discrepancy: no id: {:?}", game.id)
                    }))
                    .or_insert_with(Vec::new);
                entry.push(game);
                acc
            },
        );

    let groups_unplayed =
        unplayed_games.into_iter().fold(
            empty_groups
                .clone()
                .map(|group_id| (*group_id, Vec::new()))
                .collect::<BTreeMap<GroupId, Vec<UnplayedGroupGame>>>(),
            |mut acc, game| {
                let entry = acc
                    .entry(*game_group_map.get(&game.id).unwrap_or_else(|| {
                        panic!("game group map discrepancy: no id: {:?}", game.id)
                    }))
                    .or_insert_with(Vec::new);
                entry.push(game);
                acc
            },
        );

    let groups: Result<Groups, WwcError> = groups_played
        .into_iter()
        .zip(groups_unplayed.into_iter())
        .map(
            |((group_id_played, played), (group_id_unplayed, unplayed))| {
                assert!(group_id_played == group_id_unplayed);
                Group::try_new(unplayed, played)
                    .map(|group| (group_id_played, group))
                    .map_err(WwcError::from)
            },
        )
        .collect();

    let groups = groups?;
    info!("Retrieved {} groups", groups.len());
    Ok(Json(groups))
}

/// Get all display names
pub(crate) async fn get_display_names(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<(i32, Option<String>, String)>>, AppError> {
    let display_names = wwc_db::get_all_display_names(&pool).await?;

    info!("Retrieved {} display names", display_names.len());
    Ok(Json(display_names))
}

/// Helper function to ensure a player exists in the database
/// Creates a default player with the given ID if it doesn't exist
pub(crate) async fn ensure_player_exists(
    pool: &SqlitePool,
    player_id: PlayerId,
) -> Result<(), AppError> {
    // Check if player exists
    let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM players WHERE id = ?")
        .bind(i32::from(player_id))
        .fetch_one(pool)
        .await?;

    // If player doesn't exist, create them
    if exists == 0 {
        let player_name = format!("Player {}", i32::from(player_id));
        sqlx::query("INSERT INTO players (id, name) VALUES (?, ?)")
            .bind(i32::from(player_id))
            .bind(player_name)
            .execute(pool)
            .await?;
        info!("Auto-created player with id {}", player_id);
    }

    Ok(())
}
