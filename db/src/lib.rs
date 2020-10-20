#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use crate::models::*;
use crate::schema::games::dsl::*;
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

pub fn insert_game<'a, T: Into<NewGame<'a>>>(game: T) {
    let game = game.into();
    let connection = establish_connection();

    diesel::insert_into(games)
        .values(&game)
        .execute(&connection)
        .expect("Error saving new post");
}
