use crate::data::{clear_preds, get_groups_played_with_preds, get_teams, save_preds};
use crate::game::ScoreInput;
use crate::group::view_group_play;
use crate::UiError;
use seed::{prelude::*, *};
use wwc_core::{
    game::GameId,
    group::{
        order::{fifa_2018, Random, Rules},
        GroupId, Groups,
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
        group_rules: fifa_2018(),
    }
}

pub(crate) enum Msg {
    UrlChanged(subs::UrlChanged),
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
            log!("Fetching teams");
            orders
                .skip()
                .perform_cmd(async { Msg::TeamsFetched(get_teams().await) });
        }

        Msg::TeamsFetched(Ok(teams)) => {
            log!(&format!("Fetched {} teams", teams.len()));
            model.teams = teams;
        }

        Msg::TeamsFetched(Err(fetch_error)) => {
            error!("Error fetching teams {}", fetch_error);
        }

        Msg::FetchGroups => {
            log!("Fetching groups");
            let player_id = model.player.id();
            orders.skip().perform_cmd(async move {
                Msg::GroupsFetched(get_groups_played_with_preds(player_id).await)
            });
        }

        Msg::GroupsFetched(Ok(groups)) => {
            log!(&format!("Fetched {} groups", groups.len()));
            model.groups = groups;
        }

        Msg::GroupsFetched(Err(fetch_error)) => {
            error!("Error fetching groups {}", fetch_error);
        }

        Msg::PlayGame(input) => {
            let group = model.groups.get_mut(&input.group_id).unwrap();
            group.play_game(input.game_id, input.score);
        }
        Msg::UnplayGame(group_id, game_id) => {
            log!("Replaying game {} in group {}", game_id, group_id);
            let group = model.groups.get_mut(&group_id).unwrap();
            group.unplay_game(game_id);
        }
        Msg::SavePreds => {
            let player_preds = model_preds(&model);
            log!("Saving preds");
            orders.skip().perform_cmd(async {
                Msg::PredsSaved(save_preds(player_preds).await.map_err(UiError::from))
            });
        }
        Msg::PredsSaved(Err(fetch_error)) => {
            error!("Error saving preds {}", fetch_error);
        }
        Msg::ClearPreds => {
            log!("Clearing preds");
            model.groups.iter_mut().for_each(|(_, group)| {
                let tmp = group.clone();
                tmp.played_games().for_each(|game| group.unplay_game(game.id))
            });
            orders.skip().perform_cmd(async {
                Msg::PredsCleared(clear_preds().await.map_err(UiError::from))
            });
        }
        Msg::PredsCleared(Ok(())) => {
            log!("Preds cleared");
        }
        Msg::PredsCleared(Err(fetch_error)) => {
            error!("Error clearing preds {}", fetch_error);
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
            .map(|(_, group)| group.played_games())
            .flatten()
            .map(|game| (Prediction::from(game.clone())))
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
