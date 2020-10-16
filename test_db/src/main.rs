#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;

extern crate dotenv;

mod db;
use db::*;

fn main() {
    let posts = get_posts();
    println!("{:?}", posts);
}
