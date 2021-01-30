#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use crate::models::*;
use crate::schema::games::dsl::*;
use crate::schema::group_game_map::dsl::*;
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

fn establish_connection() -> Result<SqliteConnection, DbError> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").map_err(|_| DbError::DbUrlMissing)?;
    Ok(SqliteConnection::establish(&database_url)?)
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
            GameId::from(u8::try_from(map_.id).unwrap()),
            GroupId::from(map_.group_id_.chars().next().unwrap()),
        )
    }))
}

pub fn insert_team(team: &wwc_core::Team) -> Result<(), DbError> {
    let team = NewTeam {
        id: u8::from(team.id).into(),
        name: &String::from(team.name.clone()),
        fifa_code: &String::from(team.fifa_code.clone()),
        iso2: &String::from(team.iso2.clone()),
        rank_: u8::from(team.rank).into(),
    };

    let connection = establish_connection()?;

    diesel::insert_into(teams)
        .values(&team)
        .execute(&connection)?;
    Ok(())
}

pub fn insert_group_game_mapping(group: (GroupId, GameId)) -> Result<(), DbError> {
    let (group_id, game_id_) = group;
    let group = NewGroupGameMap {
        id: u8::from(game_id_).into(),
        group_id_: &(String::from(char::from(group_id))),
    };
    let connection = establish_connection()?;

    diesel::insert_into(group_game_map)
        .values(&group)
        .execute(&connection)?;
    Ok(())
}

pub fn insert_game<'a, T: Into<NewGame<'a>>>(game: T) -> Result<(), DbError> {
    let game = game.into();
    let connection = establish_connection()?;

    diesel::insert_into(games)
        .values(&game)
        .execute(&connection)?;
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
