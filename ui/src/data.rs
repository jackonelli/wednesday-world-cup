use crate::UiError;
use seed::prelude::*;
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
    Ok(
        Request::new(&format!("{}/{}/{}", SERVER_IP, "get_preds", player_id))
            .fetch()
            .await?
            .check_status()?
            .json()
            .await?,
    )
}

pub(crate) async fn save_preds(preds: PlayerPredictions) -> Result<(), UiError> {
    let url = format!("{}/{}", SERVER_IP, "save_preds");
    Request::new(&url)
        .method(Method::Put)
        .json(&preds)
        .expect("Could not serialise PlayerPredictions")
        .fetch().await?.check_status()?;
    Ok(())
}

pub(crate) async fn clear_preds() -> Result<(), UiError> {
    Request::new(&format!("{}/{}", SERVER_IP, "clear_preds"))
        .fetch()
        .await?
        .check_status()?;
    Ok(())
}

pub(crate) async fn get_teams() -> Result<Teams, UiError> {
    Ok(Request::new(&format!("{}/{}", SERVER_IP, "get_teams"))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await?)
}

pub(crate) async fn get_groups() -> fetch::Result<Groups> {
    Request::new(&format!("{}/{}", SERVER_IP, "get_groups"))
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
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
        .map(|(group_id, group)| group.unplayed_games().map(move |game| (game.id, *group_id)))
        .flatten()
        .collect();

    preds.iter().for_each(|pred| {
        game_group_map.get(&pred.0).map(|group_id| {
            groups.get_mut(group_id).map(|group| group.play_game(pred.0, pred.1.clone()))
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
