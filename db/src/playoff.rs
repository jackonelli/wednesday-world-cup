use crate::DbError;
use crate::models::PlayoffTeamSourceRow;
use sqlx::SqlitePool;
use std::collections::HashSet;
use wwc_core::error::WwcError;
use wwc_core::game::GameId;
use wwc_core::group::{GroupId, GroupOutcome};
use wwc_core::playoff::TeamSource;

/// Parse a TeamSource from database row fields
fn parse_team_source(
    source_type: &str,
    group_id: Option<&str>,
    outcome: Option<&str>,
    third_place_groups: Option<&str>,
    source_game_id: Option<i32>,
) -> Result<TeamSource, DbError> {
    match source_type {
        "group_outcome" => {
            let group_id = group_id
                .ok_or_else(|| DbError::Generic("Missing group_id for group_outcome".into()))?;
            let group_id = GroupId::try_from(group_id.chars().next().unwrap())
                .map_err(|e| DbError::Core(WwcError::from(e)))?;

            let outcome = outcome
                .ok_or_else(|| DbError::Generic("Missing outcome for group_outcome".into()))?;

            let group_outcome = match outcome {
                "winner" => GroupOutcome::Winner(group_id),
                "runner_up" => GroupOutcome::RunnerUp(group_id),
                "third_place" => {
                    let groups_str = third_place_groups
                        .ok_or_else(|| DbError::Generic("Missing third_place_groups".into()))?;
                    // Parse JSON array like '["A","B","C"]'
                    let groups: Vec<String> = serde_json::from_str(groups_str).map_err(|e| {
                        DbError::Generic(format!("Invalid third_place_groups JSON: {}", e))
                    })?;
                    let group_set: HashSet<GroupId> = groups
                        .into_iter()
                        .map(|s| GroupId::try_from(s.chars().next().unwrap()))
                        .collect::<Result<_, _>>()
                        .map_err(|e| DbError::Core(WwcError::from(e)))?;
                    GroupOutcome::ThirdPlace(group_set)
                }
                _ => return Err(DbError::Generic(format!("Unknown outcome: {}", outcome))),
            };

            Ok(TeamSource::GroupOutcome(group_outcome))
        }
        "winner_of" => {
            let game_id = source_game_id
                .ok_or_else(|| DbError::Generic("Missing source_game_id for winner_of".into()))?;
            Ok(TeamSource::WinnerOf(GameId::from(
                u32::try_from(game_id).unwrap(),
            )))
        }
        "loser_of" => {
            let game_id = source_game_id
                .ok_or_else(|| DbError::Generic("Missing source_game_id for loser_of".into()))?;
            Ok(TeamSource::LoserOf(GameId::from(
                u32::try_from(game_id).unwrap(),
            )))
        }
        _ => Err(DbError::Generic(format!(
            "Unknown source_type: {}",
            source_type
        ))),
    }
}

/// Get playoff team sources from the database
pub async fn get_playoff_team_sources(
    pool: &SqlitePool,
) -> Result<Vec<(GameId, (TeamSource, TeamSource))>, DbError> {
    let rows = sqlx::query_as::<_, PlayoffTeamSourceRow>(
        "SELECT * FROM playoff_team_sources ORDER BY game_id",
    )
    .fetch_all(pool)
    .await
    .map_err(DbError::Sqlx)?;

    let team_sources: Result<Vec<_>, DbError> = rows
        .into_iter()
        .map(|row| {
            let game_id = GameId::from(u32::try_from(row.game_id).unwrap());

            let home_source = parse_team_source(
                &row.home_source_type,
                row.home_group_id.as_deref(),
                row.home_outcome.as_deref(),
                row.home_third_place_groups.as_deref(),
                row.home_source_game_id,
            )?;

            let away_source = parse_team_source(
                &row.away_source_type,
                row.away_group_id.as_deref(),
                row.away_outcome.as_deref(),
                row.away_third_place_groups.as_deref(),
                row.away_source_game_id,
            )?;

            Ok((game_id, (home_source, away_source)))
        })
        .collect();

    team_sources
}

/// Insert playoff team sources into the database
pub async fn insert_playoff_team_sources(
    pool: &SqlitePool,
    team_sources: &[(GameId, (TeamSource, TeamSource))],
) -> Result<(), DbError> {
    for (game_id, (home_source, away_source)) in team_sources {
        let game_id_i32 = i32::try_from(u32::from(*game_id)).unwrap();

        // Helper to extract fields from TeamSource
        let extract_source = |source: &TeamSource| -> (
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<i32>,
        ) {
            match source {
                TeamSource::GroupOutcome(outcome) => match outcome {
                    GroupOutcome::Winner(group_id) => (
                        "group_outcome".to_string(),
                        Some(String::from(char::from(*group_id))),
                        Some("winner".to_string()),
                        None,
                        None,
                    ),
                    GroupOutcome::RunnerUp(group_id) => (
                        "group_outcome".to_string(),
                        Some(String::from(char::from(*group_id))),
                        Some("runner_up".to_string()),
                        None,
                        None,
                    ),
                    GroupOutcome::ThirdPlace(groups) => {
                        let mut group_chars: Vec<String> = groups
                            .iter()
                            .map(|g| String::from(char::from(*g)))
                            .collect();
                        group_chars.sort();
                        let groups_json = serde_json::to_string(&group_chars).unwrap();
                        (
                            "group_outcome".to_string(),
                            None,
                            Some("third_place".to_string()),
                            Some(groups_json),
                            None,
                        )
                    }
                },
                TeamSource::WinnerOf(game_id) => (
                    "winner_of".to_string(),
                    None,
                    None,
                    None,
                    Some(i32::try_from(u32::from(*game_id)).unwrap()),
                ),
                TeamSource::LoserOf(game_id) => (
                    "loser_of".to_string(),
                    None,
                    None,
                    None,
                    Some(i32::try_from(u32::from(*game_id)).unwrap()),
                ),
            }
        };

        let (
            home_source_type,
            home_group_id,
            home_outcome,
            home_third_place_groups,
            home_source_game_id,
        ) = extract_source(home_source);
        let (
            away_source_type,
            away_group_id,
            away_outcome,
            away_third_place_groups,
            away_source_game_id,
        ) = extract_source(away_source);

        sqlx::query(
            "INSERT INTO playoff_team_sources (
                game_id,
                home_source_type, home_group_id, home_outcome, home_third_place_groups, home_source_game_id,
                away_source_type, away_group_id, away_outcome, away_third_place_groups, away_source_game_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(game_id_i32)
        .bind(home_source_type)
        .bind(home_group_id)
        .bind(home_outcome)
        .bind(home_third_place_groups)
        .bind(home_source_game_id)
        .bind(away_source_type)
        .bind(away_group_id)
        .bind(away_outcome)
        .bind(away_third_place_groups)
        .bind(away_source_game_id)
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    }

    Ok(())
}

/// Clear all playoff team sources from the database
pub async fn clear_playoff_team_sources(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::query("DELETE FROM playoff_team_sources")
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    Ok(())
}
