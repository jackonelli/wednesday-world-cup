#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use itertools::Itertools;
use rocket::http::Method;
use rocket::response::status::NotFound;
use rocket_contrib::json::Json;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions, Error};
use wwc_core::group::{Group, Groups};
use wwc_core::team::Teams;

#[get("/get_teams")]
fn get_teams() -> Result<Json<Teams>, NotFound<String>> {
    let teams: Teams = wwc_db::get_teams().map(|x| (x.id, x)).collect();
    Ok(Json(teams))
}

#[get("/get_groups")]
fn get_groups() -> Result<Json<Groups>, NotFound<String>> {
    let (played_games, unplayed_games) = wwc_db::get_group_games();
    let group_game_map = wwc_db::get_group_game_maps()
        .map(|(game, group)| (group, game))
        .into_group_map();
    // TODO: Very inefficient. Better to iterate overe the games and assign to groups.
    Ok(Json(group_game_map.iter().fold(
        Groups::new(),
        |mut acc, (id, games)| {
            let group = Group::try_new(
                unplayed_games
                    .iter()
                    .filter(|x| games.contains(&x.id))
                    .cloned()
                    .collect(),
                played_games
                    .iter()
                    .filter(|x| games.contains(&x.id))
                    .cloned()
                    .collect(),
            )
            .unwrap();
            acc.insert(*id, group);
            acc
        },
    )))
}

fn make_cors() -> Cors {
    let allowed_origins = AllowedOrigins::some_exact(&[
        "http://129.16.37.14:8888",
        "http://192.168.0.15:8888",
        "http://localhost:8888",
    ]);

    CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&[
            "Authorization",
            "Accept",
            "Access-Control-Allow-Origin",
        ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error while building CORS")
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![get_teams, get_groups])
        .attach(make_cors())
}

fn main() -> Result<(), Error> {
    rocket().launch();
    Ok(())
}
