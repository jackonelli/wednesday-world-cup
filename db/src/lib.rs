#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use crate::models::*;
use crate::schema::games::dsl::*;
use crate::schema::group_game_map::dsl::*;
use crate::schema::teams::dsl::*;
use diesel::prelude::*;
use dotenv::dotenv;
use itertools::{Either, Itertools};
use std::convert::TryFrom;
use std::env;
use wwc_core::group::{
    game::{GroupGameId, PlayedGroupGame, UnplayedGroupGame},
    GroupId,
};

fn establish_connection() -> SqliteConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn get_games() -> Vec<Game> {
    let connection = establish_connection();
    games
        .load::<Game>(&connection)
        .expect("Error loading posts")
}

pub fn get_group_games() -> (Vec<PlayedGroupGame>, Vec<UnplayedGroupGame>) {
    let connection = establish_connection();
    let group_games = games
        .filter(type_.eq("group"))
        .load::<Game>(&connection)
        .expect("Error loading posts");

    group_games.into_iter().partition_map(|game| {
        if game.played {
            Either::Left(game.into())
        } else {
            Either::Right(game.into())
        }
    })
}

pub fn get_teams() -> impl Iterator<Item = wwc_core::Team> {
    let connection = establish_connection();
    let db_teams = teams
        .load::<Team>(&connection)
        .expect("Error loading posts");
    db_teams.into_iter().map(|team| team.into())
}

pub fn get_group_game_maps() -> impl Iterator<Item = (GroupGameId, GroupId)> {
    let connection = establish_connection();
    let db_teams = group_game_map
        .load::<GroupGameMap>(&connection)
        .expect("Error loading posts");
    db_teams.into_iter().map(|map_| {
        (
            GroupGameId::from(u8::try_from(map_.game_id).unwrap()),
            GroupId::from(map_.group_id_.chars().next().unwrap()),
        )
    })
}

pub fn insert_team(team: &wwc_core::Team) {
    let team = NewTeam {
        id: u8::from(team.id).into(),
        name: &String::from(team.name.clone()),
        fifa_code: &String::from(team.fifa_code.clone()),
        iso2: &String::from(team.iso2.clone()),
        rank_: u8::from(team.rank).into(),
    };

    let connection = establish_connection();

    diesel::insert_into(teams)
        .values(&team)
        .execute(&connection)
        .expect("Error saving new post");
}

pub fn insert_group_game_mapping(group: (GroupId, GroupGameId)) {
    let (group_id, game_id_) = group;
    let group = NewGroupGameMap {
        game_id: u8::from(game_id_).into(),
        group_id_: &(String::from(char::from(group_id))),
    };
    let connection = establish_connection();

    diesel::insert_into(group_game_map)
        .values(&group)
        .execute(&connection)
        .expect("Error saving new post");
}

pub fn insert_game<'a, T: Into<NewGame<'a>>>(game: T) {
    let game = game.into();
    let connection = establish_connection();

    diesel::insert_into(games)
        .values(&game)
        .execute(&connection)
        .expect("Error saving new post");
}

pub fn clear_teams() {
    let connection = establish_connection();
    diesel::delete(teams)
        .execute(&connection)
        .expect("Could not clear table");
}
pub fn clear_games() {
    let connection = establish_connection();
    diesel::delete(games)
        .execute(&connection)
        .expect("Could not clear table");
}
pub fn clear_group_game_maps() {
    let connection = establish_connection();
    diesel::delete(group_game_map)
        .execute(&connection)
        .expect("Could not clear table");
}
