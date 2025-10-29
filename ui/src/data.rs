use crate::UiError;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wwc_core::player::{PlayerId, Prediction};
use wwc_core::playoff::TeamSource;
use wwc_core::{
    game::GameId,
    group::{GroupId, Groups},
    player::PlayerPredictions,
    team::Teams,
};

const SERVER_IP: &str = "http://localhost:8000";

#[derive(Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
    player_id: i32,
    display_name: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
struct MeResponse {
    player_id: i32,
    display_name: String,
    bot_name: Option<String>,
}

/// Get current user info from token
pub(crate) async fn get_me(token: &str) -> Result<(String, Option<String>), UiError> {
    let url = format!("{}/{}", SERVER_IP, "me");
    let response = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;

    if response.ok() {
        let me_response: MeResponse = response.json().await?;
        Ok((me_response.display_name, me_response.bot_name))
    } else {
        let error_response: ErrorResponse = response.json().await?;
        Err(UiError::Server(error_response.error))
    }
}

/// Login with username and password, returns (token, player_id, display_name)
pub(crate) async fn login(
    username: &str,
    password: &str,
) -> Result<(String, i32, String), UiError> {
    let url = format!("{}/{}", SERVER_IP, "login");
    let login_req = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
    };
    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(&login_req)?
        .send()
        .await?;

    if response.ok() {
        let login_response: LoginResponse = response.json().await?;
        Ok((
            login_response.token,
            login_response.player_id,
            login_response.display_name,
        ))
    } else {
        let error_response: ErrorResponse = response.json().await?;
        Err(UiError::Server(error_response.error))
    }
}

pub(crate) async fn get_preds(player_id: PlayerId) -> Result<Vec<Prediction>, UiError> {
    let response = Request::get(&format!("{}/{}/{}", SERVER_IP, "get_preds", player_id))
        .send()
        .await?;
    Ok(response.json().await?)
}

pub(crate) async fn save_preds(preds: PlayerPredictions, token: &str) -> Result<(), UiError> {
    let url = format!("{}/{}", SERVER_IP, "save_preds");
    let json_body = serde_json::to_string(&preds)?;
    Request::put(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .body(json_body)?
        .send()
        .await?;
    Ok(())
}

pub(crate) async fn clear_my_preds(token: &str) -> Result<(), UiError> {
    Request::get(&format!("{}/{}", SERVER_IP, "clear_my_preds"))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await?;
    Ok(())
}

pub(crate) async fn clear_preds(admin_secret: &str) -> Result<(), UiError> {
    Request::get(&format!("{}/{}", SERVER_IP, "clear_preds"))
        .header("Authorization", &format!("Bearer {}", admin_secret))
        .send()
        .await?;
    Ok(())
}

pub(crate) async fn get_teams() -> Result<Teams, UiError> {
    let response = Request::get(&format!("{}/{}", SERVER_IP, "get_teams"))
        .send()
        .await?;
    Ok(response.json().await?)
}

pub(crate) async fn get_groups() -> Result<Groups, UiError> {
    let response = Request::get(&format!("{}/{}", SERVER_IP, "get_groups"))
        .send()
        .await?;
    response.json().await.map_err(Into::into)
}

/// Fetches all group games and unplays them to simulate settings at betting time.
pub(crate) async fn get_groups_played_with_preds(player_id: PlayerId) -> Result<Groups, UiError> {
    let preds = get_preds(player_id).await?;
    let mut groups = get_groups().await?;
    groups.iter_mut().for_each(|(_, group)| {
        // TODO: How to remove this allocation?
        let tmp = group.clone();
        tmp.played_games()
            .for_each(|game| group.unplay_game(game.id))
    });
    let game_group_map: HashMap<GameId, GroupId> = groups
        .iter()
        .flat_map(|(group_id, group)| group.unplayed_games().map(move |game| (game.id, *group_id)))
        .collect();

    preds.iter().for_each(|pred| {
        game_group_map.get(&pred.0).map(|group_id| {
            groups
                .get_mut(group_id)
                .map(|group| group.play_game(pred.0, pred.1))
        });
    });
    Ok(groups)
}

/// Fetches all group games and unplays them to simulate settings at betting time.
pub(crate) async fn get_groups_as_unplayed() -> Result<Groups, UiError> {
    let mut groups = get_groups().await?;
    groups.iter_mut().for_each(|(_, group)| {
        // TODO: How to remove this allocation?
        let tmp = group.clone();
        tmp.played_games()
            .for_each(|game| group.unplay_game(game.id))
    });
    Ok(groups)
}

/// Fetch playoff team sources from server
pub(crate) async fn get_playoff_team_sources()
-> Result<Vec<(GameId, (TeamSource, TeamSource))>, UiError> {
    let response = Request::get(&format!("{}/{}", SERVER_IP, "get_playoff_team_sources"))
        .send()
        .await?;
    response.json().await.map_err(Into::into)
}
