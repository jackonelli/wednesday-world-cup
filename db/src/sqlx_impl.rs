use crate::DbError;
use dotenv::dotenv;
use itertools::{Either, Itertools};
use sqlx::{FromRow, SqlitePool};
use std::env;
use wwc_core::error::WwcError;
use wwc_core::fair_play::FairPlayScore;
use wwc_core::game::GameId;
use wwc_core::group::{
    GroupId,
    game::{GroupGameScore, PlayedGroupGame, UnplayedGroupGame},
};
use wwc_core::player::{PlayerId, PlayerPredictions, Prediction};
use wwc_core::team::{FifaCode, TeamId, TeamName, TeamRank};

// Database models
#[derive(Debug, FromRow)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub fifa_code: String,
    pub rank_: i32,
}

impl From<Team> for wwc_core::Team {
    fn from(db_team: Team) -> wwc_core::Team {
        let id = TeamId(u32::try_from(db_team.id).unwrap());
        let name = TeamName::from(db_team.name);
        let fifa_code = FifaCode::try_from(db_team.fifa_code).unwrap();
        let rank = TeamRank(u32::try_from(db_team.rank_).unwrap());
        wwc_core::Team {
            id,
            name,
            fifa_code,
            rank,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct Game {
    pub id: i32,
    pub type_: String,
    pub home_team: i32,
    pub away_team: i32,
    pub home_result: Option<i32>,
    pub away_result: Option<i32>,
    pub home_penalty: Option<i32>,
    pub away_penalty: Option<i32>,
    pub home_fair_play: Option<i32>,
    pub away_fair_play: Option<i32>,
    pub played: bool,
}

impl TryFrom<Game> for PlayedGroupGame {
    type Error = DbError;
    fn try_from(game: Game) -> Result<Self, Self::Error> {
        Ok(UnplayedGroupGame::try_new(
            u32::try_from(game.id).unwrap(),
            u32::try_from(game.home_team).unwrap(),
            u32::try_from(game.away_team).unwrap(),
            wwc_core::Date::mock(),
        )
        .map_err(WwcError::from)
        .map_err(DbError::from)?
        .play(
            GroupGameScore::from((
                u32::try_from(game.home_result.unwrap()).unwrap(),
                u32::try_from(game.away_result.unwrap()).unwrap(),
            )),
            FairPlayScore::default(),
        ))
    }
}

impl TryFrom<Game> for UnplayedGroupGame {
    type Error = DbError;
    fn try_from(game: Game) -> Result<Self, Self::Error> {
        UnplayedGroupGame::try_new(
            u32::try_from(game.id).unwrap(),
            u32::try_from(game.home_team).unwrap(),
            u32::try_from(game.away_team).unwrap(),
            wwc_core::Date::mock(),
        )
        .map_err(WwcError::from)
        .map_err(DbError::from)
    }
}

#[derive(Debug, FromRow)]
pub struct GroupGameMap {
    pub id: i32,
    pub group_id_: String,
}

#[derive(Debug, FromRow)]
pub struct Pred {
    pub id: i32,
    pub player_id: i32,
    pub game_id: i32,
    pub home_result: i32,
    pub away_result: i32,
}

impl From<Pred> for Prediction {
    fn from(pred: Pred) -> Prediction {
        let score = GroupGameScore::from((
            u32::try_from(pred.home_result).unwrap(),
            u32::try_from(pred.away_result).unwrap(),
        ));
        Prediction(GameId::from(u32::try_from(pred.game_id).unwrap()), score)
    }
}

#[derive(Debug, FromRow)]
pub struct Player {
    pub id: i32,
    pub name: String,
}

// Database connection pool
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

    Ok(pool)
}

// Database operations
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

pub async fn get_preds(pool: &SqlitePool, player_id: PlayerId) -> Result<Vec<Prediction>, DbError> {
    let player_id = i32::from(player_id);
    let db_preds = sqlx::query_as::<_, Pred>("SELECT * FROM preds WHERE player_id = ?")
        .bind(player_id)
        .fetch_all(pool)
        .await
        .map_err(DbError::Sqlx)?;

    Ok(db_preds.into_iter().map(Prediction::from).collect())
}

pub async fn get_players(pool: &SqlitePool) -> Result<Vec<Player>, DbError> {
    let players = sqlx::query_as::<_, Player>("SELECT * FROM players")
        .fetch_all(pool)
        .await
        .map_err(DbError::Sqlx)?;

    Ok(players)
}

pub async fn get_games(pool: &SqlitePool) -> Result<Vec<Game>, DbError> {
    let games = sqlx::query_as::<_, Game>("SELECT * FROM games")
        .fetch_all(pool)
        .await
        .map_err(DbError::Sqlx)?;

    Ok(games)
}

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

pub async fn get_teams(pool: &SqlitePool) -> Result<Vec<wwc_core::Team>, DbError> {
    let db_teams = sqlx::query_as::<_, Team>("SELECT * FROM teams")
        .fetch_all(pool)
        .await
        .map_err(DbError::Sqlx)?;

    Ok(db_teams.into_iter().map(|team| team.into()).collect())
}

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

pub async fn insert_preds(pool: &SqlitePool, preds: &PlayerPredictions) -> Result<(), DbError> {
    let player_id = i32::from(preds.id);

    // Delete existing predictions
    sqlx::query("DELETE FROM preds WHERE player_id = ?")
        .bind(player_id)
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;

    // Insert new predictions
    for pred in preds.preds() {
        sqlx::query(
            "INSERT INTO preds (player_id, game_id, home_result, away_result) VALUES (?, ?, ?, ?)",
        )
        .bind(player_id)
        .bind(i32::try_from(u32::from(pred.0)).unwrap())
        .bind(i32::try_from(u32::from(pred.1.home)).unwrap())
        .bind(i32::try_from(u32::from(pred.1.away)).unwrap())
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    }

    Ok(())
}

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

pub async fn clear_players(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::query("DELETE FROM players")
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    Ok(())
}

pub async fn clear_preds(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::query("DELETE FROM preds")
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    Ok(())
}

pub async fn clear_teams(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::query("DELETE FROM teams")
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    Ok(())
}

pub async fn clear_games(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::query("DELETE FROM games")
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    Ok(())
}

pub async fn clear_group_game_maps(pool: &SqlitePool) -> Result<(), DbError> {
    sqlx::query("DELETE FROM group_game_map")
        .execute(pool)
        .await
        .map_err(DbError::Sqlx)?;
    Ok(())
}
