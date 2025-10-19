use crate::UiError;
use gloo_net::http::Request;
use std::collections::HashMap;
use wwc_core::player::{PlayerId, Prediction};
use wwc_core::{
    game::GameId,
    group::{GroupId, Groups},
    player::PlayerPredictions,
    team::Teams,
};

const SERVER_IP: &str = "http://localhost:8000";

pub(crate) async fn get_preds(player_id: PlayerId) -> Result<Vec<Prediction>, UiError> {
    let response = Request::get(&format!("{}/{}/{}", SERVER_IP, "get_preds", player_id))
        .send()
        .await?;
    Ok(response.json().await?)
}

pub(crate) async fn save_preds(preds: PlayerPredictions) -> Result<(), UiError> {
    let url = format!("{}/{}", SERVER_IP, "save_preds");
    let json_body = serde_json::to_string(&preds)?;
    Request::put(&url)
        .header("Content-Type", "application/json")
        .body(json_body)?
        .send()
        .await?;
    Ok(())
}

pub(crate) async fn clear_preds() -> Result<(), UiError> {
    Request::get(&format!("{}/{}", SERVER_IP, "clear_preds"))
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
