#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use crate::models::*;
use crate::schema::games::dsl::*;
use crate::schema::group_game_map::dsl::*;
use crate::schema::players::dsl::name as player_name;
use crate::schema::players::dsl::players;
use crate::schema::preds::dsl::*;
use crate::schema::teams::dsl::*;
use diesel::prelude::*;
use diesel::result::ConnectionError;
use diesel::result::Error as QueryError;
use dotenv::dotenv;
use itertools::{Either, Itertools};
use std::convert::TryFrom;
use std::env;
use thiserror::Error;
use wwc_core::game::GameId;
use wwc_core::group::{
    game::{PlayedGroupGame, UnplayedGroupGame},
    GroupId,
};
use wwc_core::player::{PlayerId, PlayerPredictions, Prediction};

fn establish_connection() -> Result<SqliteConnection, DbError> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").map_err(|_| DbError::DbUrlMissing)?;
    Ok(SqliteConnection::establish(&database_url)?)
}

pub fn register_player(name_: &str) -> Result<(), DbError> {
    let connection = establish_connection()?;
    let db_players = players
        .filter(player_name.eq(name_))
        .load::<Player>(&connection)?;
    let player = NewPlayer { name: name_ };
    if db_players.is_empty() {
        diesel::insert_into(players)
            .values(&player)
            .execute(&connection)?;
        Ok(())
    } else {
        Err(DbError::Generic(format!(
            "Player with name: '{}' already in db",
            name_
        )))
    }
}

pub fn get_preds(player_id_: PlayerId) -> Result<Vec<Prediction>, DbError> {
    let connection = establish_connection()?;
    let player_id_ = i32::from(player_id_);
    let db_preds = preds
        .filter(player_id.eq(player_id_))
        .load::<Pred>(&connection)?;
    Ok(db_preds.into_iter().map(Prediction::from).collect())
}

pub fn get_players() -> Result<Vec<Player>, DbError> {
    let connection = establish_connection()?;
    Ok(players.load::<Player>(&connection)?)
}

pub fn get_games() -> Result<Vec<Game>, DbError> {
    let connection = establish_connection()?;
    Ok(games.load::<Game>(&connection)?)
}

pub fn get_group_games() -> Result<(Vec<PlayedGroupGame>, Vec<UnplayedGroupGame>), DbError> {
    let connection = establish_connection()?;
    let group_games = games.filter(type_.eq("group")).load::<Game>(&connection)?;

    Ok(group_games.into_iter().partition_map(|game| {
        if game.played {
            Either::Left(game.into())
        } else {
            Either::Right(game.into())
        }
    }))
}

pub fn get_teams() -> Result<impl Iterator<Item = wwc_core::Team>, DbError> {
    let connection = establish_connection()?;
    let db_teams = teams.load::<Team>(&connection)?;
    Ok(db_teams.into_iter().map(|team| team.into()))
}

pub fn get_group_game_maps() -> Result<impl Iterator<Item = (GameId, GroupId)>, DbError> {
    let connection = establish_connection()?;
    let db_teams = group_game_map.load::<GroupGameMap>(&connection)?;
    Ok(db_teams.into_iter().map(|map_| {
        (
            GameId::from(u32::try_from(map_.id).unwrap()),
            GroupId::from(map_.group_id_.chars().next().unwrap()),
        )
    }))
}

pub fn insert_preds(preds_: &PlayerPredictions) -> Result<(), DbError> {
    let connection = establish_connection()?;
    let player_id_ = i32::from(preds_.id);
    diesel::delete(preds.filter(player_id.eq(player_id_))).execute(&connection)?;
    let preds_: Vec<NewPred> = preds_
        .preds()
        .map(move |pred| NewPred::from(&(preds_.id, pred.clone())))
        .collect();
    diesel::insert_into(preds)
        .values(&preds_)
        .execute(&connection)?;
    Ok(())
}

pub fn insert_teams(teams_: &[wwc_core::Team]) -> Result<(), DbError> {
    let teams_: Vec<NewTeam> = teams_.iter().map(NewTeam::from).collect();

    let connection = establish_connection()?;

    diesel::insert_into(teams)
        .values(&teams_)
        .execute(&connection)?;
    Ok(())
}

pub fn insert_games<'a, T: 'a>(games_: &'a [T]) -> Result<(), DbError>
where
    &'a T: Into<NewGame<'a>>,
{
    let games_: Vec<NewGame> = games_.iter().map(|game| game.into()).collect();
    let connection = establish_connection()?;
    diesel::insert_into(games)
        .values(&games_)
        .execute(&connection)?;
    Ok(())
}

pub fn insert_group_game_mappings(group_mappings: &[(GroupId, GameId)]) -> Result<(), DbError> {
    let mappings: Vec<_> = group_mappings
        .iter()
        .map(|(group_id, game_id_)| (String::from(char::from(*group_id)), *game_id_))
        .collect();
    let mappings: Vec<_> = mappings.iter().map(NewGroupGameMap::from).collect();
    let connection = establish_connection()?;
    diesel::insert_into(group_game_map)
        .values(&mappings)
        .execute(&connection)?;
    Ok(())
}

pub fn clear_players() -> Result<(), DbError> {
    let connection = establish_connection()?;
    diesel::delete(players)
        .execute(&connection)
        .expect("Could not clear table");
    Ok(())
}

pub fn clear_preds() -> Result<(), DbError> {
    let connection = establish_connection()?;
    diesel::delete(preds)
        .execute(&connection)
        .expect("Could not clear table");
    Ok(())
}

pub fn clear_teams() -> Result<(), DbError> {
    let connection = establish_connection()?;
    diesel::delete(teams)
        .execute(&connection)
        .expect("Could not clear table");
    Ok(())
}

pub fn clear_games() -> Result<(), DbError> {
    let connection = establish_connection()?;
    diesel::delete(games)
        .execute(&connection)
        .expect("Could not clear table");
    Ok(())
}

pub fn clear_group_game_maps() -> Result<(), DbError> {
    let connection = establish_connection()?;
    diesel::delete(group_game_map)
        .execute(&connection)
        .expect("Could not clear table");
    Ok(())
}

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Missing 'DATABASE_URL'")]
    DbUrlMissing,
    #[error("Database connection: {0}")]
    Connection(#[from] ConnectionError),
    #[error("Database query: {0}")]
    Query(#[from] QueryError),
    #[error("Could you be more specific: {0}")]
    Generic(String),
}
