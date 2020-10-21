#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use crate::models::*;
use crate::schema::games::dsl::*;
use crate::schema::teams::dsl::*;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

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

pub fn get_teams() -> impl Iterator<Item = wwc_core::Team> {
    let connection = establish_connection();
    let db_teams = teams
        .load::<Team>(&connection)
        .expect("Error loading posts");
    db_teams.into_iter().map(|team| team.into())
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

pub fn insert_game<'a, T: Into<NewGame<'a>>>(game: T) {
    let game = game.into();
    let connection = establish_connection();

    diesel::insert_into(games)
        .values(&game)
        .execute(&connection)
        .expect("Error saving new post");
}
