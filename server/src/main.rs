#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use itertools::Itertools;
use rocket::http::Method;
use rocket::response::status::BadRequest;
use rocket_contrib::json::Json;
use rocket_cors::{AllowedOrigins, Cors, CorsOptions};
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;
use wwc_core::error::WwcError;
use wwc_core::game::GameId;
use wwc_core::group::{game::PlayedGroupGame, game::UnplayedGroupGame, Group, GroupId, Groups};
use wwc_core::player::{PlayerId, PlayerPredictions, Prediction};
use wwc_core::team::Teams;

/// Save preds
#[post("/save_preds", data = "<player_preds>", format = "application/json")]
fn save_preds(player_preds: Json<PlayerPredictions>) -> Result<(), BadRequest<String>> {
    let player_preds = player_preds.into_inner();
    println!("Preds:\n{:?}", player_preds);
    let tmp = wwc_db::insert_preds(&player_preds)
        .map_err(ServerError::from)
        .map_err(BadRequest::from);
    tmp?;
    Ok(())
}

/// Get teams
#[get("/get_teams")]
fn get_teams() -> Result<Json<Teams>, BadRequest<String>> {
    let teams: Teams = wwc_db::get_teams()
        .map_err(ServerError::from)
        .map_err(BadRequest::from)?
        .map(|x| (x.id, x))
        .collect();
    Ok(Json(teams))
}

/// Get predictions
#[get("/get_preds/<player_id>")]
fn get_preds(player_id: i32) -> Result<Json<Vec<Prediction>>, BadRequest<String>> {
    let preds = wwc_db::get_preds(PlayerId::from(player_id))
        .map_err(ServerError::from)
        .map_err(BadRequest::from)?;
    Ok(Json(preds))
}

/// Clear predictions
#[get("/clear_preds")]
fn clear_preds() -> Result<(), BadRequest<String>> {
    let preds = wwc_db::clear_preds()
        .map_err(ServerError::from)
        .map_err(BadRequest::from)?;
    println!("Clear preds res {:?}", preds);
    Ok(())
}

/// Get groups
///
/// Loads group games and a GameId: GroupId map from the db
/// The games (played and unplayed) games are then mapped to prospective groups.
/// The final groups are validated (with a fallible constructor) and collected together.
#[get("/get_groups")]
fn get_groups() -> Result<Json<Groups>, BadRequest<String>> {
    let (played_games, unplayed_games) = wwc_db::get_group_games()
        .map_err(ServerError::from)
        .map_err(BadRequest::from)?;
    let game_group_map = wwc_db::get_group_game_maps()
        .map_err(ServerError::from)
        .map_err(BadRequest::from)?
        .collect::<HashMap<GameId, GroupId>>();

    let empty_groups = game_group_map
        .iter()
        .map(|(_game_id, group_id)| group_id)
        .unique();

    let groups_played =
        played_games.into_iter().fold(
            empty_groups
                .clone()
                .map(|group_id| (*group_id, Vec::new()))
                .collect::<BTreeMap<GroupId, Vec<PlayedGroupGame>>>(),
            |mut acc, game| {
                let entry = acc
                    .entry(*game_group_map.get(&game.id).unwrap_or_else(|| {
                        panic!("game group map discrepancy: no id: {:?}", game.id)
                    }))
                    .or_insert_with(Vec::new);
                entry.push(game);
                acc
            },
        );

    let groups_unplayed =
        unplayed_games.into_iter().fold(
            empty_groups
                .clone()
                .map(|group_id| (*group_id, Vec::new()))
                .collect::<BTreeMap<GroupId, Vec<UnplayedGroupGame>>>(),
            |mut acc, game| {
                let entry = acc
                    .entry(*game_group_map.get(&game.id).unwrap_or_else(|| {
                        panic!("game group map discrepancy: no id: {:?}", game.id)
                    }))
                    .or_insert_with(Vec::new);
                entry.push(game);
                acc
            },
        );

    let groups: Result<Groups, WwcError> = groups_played
        .into_iter()
        .zip(groups_unplayed.into_iter())
        .map(
            |((group_id_played, played), (group_id_unplayed, unplayed))| {
                assert!(group_id_played == group_id_unplayed);
                Group::try_new(unplayed, played)
                    .map(|group| (group_id_played, group))
                    .map_err(WwcError::from)
            },
        )
        .collect();

    Ok(Json(
        groups
            .map_err(ServerError::from)
            .map_err(BadRequest::from)?,
    ))
}

fn make_cors() -> Cors {
    let allowed_origins = AllowedOrigins::some_exact(&[
        "http://129.16.37.14:8888",
        "http://192.168.0.15:8888",
        "http://localhost:8888",
    ]);

    CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post]
            .into_iter()
            .map(From::from)
            .collect(),
        // allowed_headers: AllowedHeaders::some(&[
        //     "Authorization",
        //     "Accept",
        //     "Access-Control-Allow-Origin",
        // ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error while building CORS")
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount(
            "/",
            routes![get_teams, get_groups, save_preds, get_preds, clear_preds],
        )
        .attach(make_cors())
}

fn main() {
    rocket().launch();
}

#[derive(Error, Debug)]
enum ServerError {
    #[error("Database error: {0}")]
    Db(#[from] wwc_db::DbError),
    #[error("Wwc core error: {0}")]
    Wwc(#[from] WwcError),
}

impl From<ServerError> for BadRequest<String> {
    fn from(server_err: ServerError) -> Self {
        BadRequest(Some(server_err.to_string()))
    }
}
