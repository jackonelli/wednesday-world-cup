use crate::UiError;
use crate::data::{clear_preds, get_groups_played_with_preds, get_teams, save_preds};
use crate::game::ScoreInput;
use crate::group::view_group_play;
use seed::{prelude::*, *};
use web_sys::console;
use wwc_core::{
    game::GameId,
    group::{
        GroupId, Groups,
        order::{Random, Rules, fifa_2018_rules},
    },
    player::{Player, PlayerPredictions, Prediction},
    team::Teams,
};
const ENTER_KEY: &str = "Enter";
const ESCAPE_KEY: &str = "Escape";

struct Model {
    groups: Groups,
    player: Player,
    teams: Teams,
    base_url: Url,
    group_rules: Rules<Random>,
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.perform_cmd(async { Msg::FetchTeams });
    orders.perform_cmd(async { Msg::FetchGroups });
    Model {
        groups: Groups::new(),
        player: Player::dummy(),
        teams: Teams::new(),
        base_url: Url::new(),
        group_rules: fifa_2018_rules(),
    }
}

pub(crate) enum Msg {
    FetchTeams,
    TeamsFetched(Result<Teams, UiError>),
    FetchGroups,
    GroupsFetched(Result<Groups, UiError>),
    PlayGame(ScoreInput),
    UnplayGame(GroupId, GameId),
    SavePreds,
    PredsSaved(Result<(), UiError>),
    ClearPreds,
    PredsCleared(Result<(), UiError>),
    UnfinishedScoreInput,
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::FetchTeams => {
            console::log_1(&"Fetching teams".into());
            orders
                .skip()
                .perform_cmd(async { Msg::TeamsFetched(get_teams().await) });
        }

        Msg::TeamsFetched(Ok(teams)) => {
            console::log_1(&format!("Fetched {} teams", teams.len()).into());
            model.teams = teams;
        }

        Msg::TeamsFetched(Err(fetch_error)) => {
            console::error_1(&format!("Error fetching teams {}", fetch_error).into());
        }

        Msg::FetchGroups => {
            console::log_1(&"Fetching groups".into());
            let player_id = model.player.id();
            orders.skip().perform_cmd(async move {
                Msg::GroupsFetched(get_groups_played_with_preds(player_id).await)
            });
        }

        Msg::GroupsFetched(Ok(groups)) => {
            console::log_1(&format!("Fetched {} groups", groups.len()).into());
            model.groups = groups;
        }

        Msg::GroupsFetched(Err(fetch_error)) => {
            console::error_1(&format!("Error fetching groups {}", fetch_error).into());
        }

        Msg::PlayGame(input) => {
            let group = model.groups.get_mut(&input.group_id).unwrap();
            group.play_game(input.game_id, input.score);
        }
        Msg::UnplayGame(group_id, game_id) => {
            console::log_1(&format!("Replaying game {} in group {}", game_id, group_id).into());
            let group = model.groups.get_mut(&group_id).unwrap();
            group.unplay_game(game_id);
        }
        Msg::SavePreds => {
            let player_preds = model_preds(model);
            console::log_1(&"Saving preds".into());
            orders.skip().perform_cmd(async {
                Msg::PredsSaved(save_preds(player_preds).await.map_err(UiError::from))
            });
        }
        Msg::PredsSaved(Err(fetch_error)) => {
            console::error_1(&format!("Error saving preds {}", fetch_error).into());
        }
        Msg::ClearPreds => {
            console::log_1(&"Clearing preds".into());
            model.groups.iter_mut().for_each(|(_, group)| {
                let tmp = group.clone();
                tmp.played_games()
                    .for_each(|game| group.unplay_game(game.id))
            });
            orders.skip().perform_cmd(async {
                Msg::PredsCleared(clear_preds().await.map_err(UiError::from))
            });
        }
        Msg::PredsCleared(Ok(())) => {
            console::log_1(&"Preds cleared".into());
        }
        Msg::PredsCleared(Err(fetch_error)) => {
            console::error_1(&format!("Error clearing preds {}", fetch_error).into());
        }
        _ => {}
    }
}

fn model_preds(model: &Model) -> PlayerPredictions {
    PlayerPredictions::new(
        model.player.id(),
        model
            .groups
            .iter()
            .flat_map(|(_, group)| group.played_games())
            .map(|game| Prediction::from(*game))
            .collect(),
    )
}

fn view(model: &Model) -> Vec<Node<Msg>> {
    nodes![
        view_header(),
        button!["Save preds", ev(Ev::Click, |_| Msg::SavePreds),]
        br!(),
        button!["Clear preds", ev(Ev::Click, |_| Msg::ClearPreds),]
        view_group_play(&model.groups, &model.teams, &model.group_rules),
        // view_play_off(&model.playoff, &model.teams)
    ]
}

fn view_header() -> Node<Msg> {
    header![C!["header"], h1!["Wednesday world cup"],]
}

#[wasm_bindgen(start)]
pub fn start() {
    let root_element = document()
        .get_elements_by_class_name("tournament")
        .item(0)
        .expect("element with the class `tournament`");

    App::start(root_element, init, update, view);
}
