#[macro_use]
extern crate diesel;

extern crate dotenv;

pub mod models;
pub mod schema;

use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

use crate::models::*;
use crate::schema::games::dsl::*;

fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn get_posts() -> Vec<Game> {
    let connection = establish_connection();
    games
        .filter(played.eq(true))
        .limit(5)
        .load::<Game>(&connection)
        .expect("Error loading posts")
}
