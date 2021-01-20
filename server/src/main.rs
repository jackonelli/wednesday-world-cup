#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use itertools::Itertools;
use rocket::http::Method;
use rocket::response::status::BadRequest;
use rocket_contrib::json::Json;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};
use thiserror::Error;
use wwc_core::error::WwcError;
use wwc_core::group::{Group, Groups};
use wwc_core::team::Teams;

#[get("/get_teams")]
fn get_teams() -> Result<Json<Teams>, BadRequest<ServerError>> {
    let teams: Teams = wwc_db::get_teams()
        .map_err(|err| BadRequest(Some(ServerError::from(err))))?
        .map(|x| (x.id, x))
        .collect();
    Ok(Json(teams))
}

#[get("/get_groups")]
fn get_groups() -> Result<Json<Groups>, BadRequest<ServerError>> {
    let (played_games, unplayed_games) =
        wwc_db::get_group_games().map_err(|err| BadRequest(Some(ServerError::from(err))))?;
    let group_game_map = wwc_db::get_group_game_maps()
        .map_err(|err| BadRequest(Some(ServerError::from(err))))?
        .map(|(game, group)| (group, game))
        .into_group_map();
    // TODO: Very inefficient. Better to iterate overe the games and assign to groups.
    Ok(Json(group_game_map.iter().try_fold(
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
            .map_err(|err| BadRequest(Some(ServerError::from(WwcError::from(err)))))?;
            acc.insert(*id, group);
            Ok(acc)
        },
    )?))
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

fn main() {
    rocket().launch();
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Database error: {0}")]
    Db(#[from] wwc_db::DbError),
    #[error("Wwc core error: {0}")]
    Wwc(#[from] WwcError),
}
