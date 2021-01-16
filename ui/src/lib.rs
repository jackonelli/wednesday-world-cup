#![allow(clippy::wildcard_imports)]
#![allow(dead_code, unused_variables)]
use crate::format::Format;
use crate::game::ScoreInput;
use seed::{prelude::*, *};
use std::collections::BTreeMap;
use wwc_core::{
    group::{
        game::GroupGameId,
        order::{fifa_2018, order_group, Random, Rules, Tiebreaker},
        Group, GroupId, Groups,
    },
    team::{Team, Teams},
};

mod format;
mod game;
mod table;
use table::DisplayTable;
const ENTER_KEY: &str = "Enter";
const ESCAPE_KEY: &str = "Escape";

pub type GroupGameMap = BTreeMap<GroupId, Vec<GroupGameId>>;

fn format_team_flag(team: &Team) -> Node<Msg> {
    span![C![format!(
        "tournament-group__flag flag-icon flag-icon-{}",
        team.iso2
    )]]
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.perform_cmd(async { Msg::FetchTeams });
    orders.perform_cmd(async { Msg::FetchGroups });
    Model {
        groups: Groups::new(),
        teams: Teams::new(),
        base_url: Url::new(),
        group_rules: fifa_2018(),
    }
}

struct Model {
    groups: Groups,
    teams: Teams,
    base_url: Url,
    group_rules: Rules<Random>,
}

pub(crate) enum Msg {
    UrlChanged(subs::UrlChanged),
    FetchTeams,
    TeamsFetched(fetch::Result<Teams>),
    FetchGroups,
    GroupsFetched(fetch::Result<Groups>),
    PlayGame(ScoreInput),
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
            log!("Error fetching teams {}", fetch_error);
        }

        Msg::FetchGroups => {
            log!("Fetching groups");
            orders
                .skip()
                .perform_cmd(async { Msg::GroupsFetched(get_groups().await) });
        }

        Msg::GroupsFetched(Ok(groups)) => {
            log!(&format!("Fetched {} groups", groups.len()));
            model.groups = groups;
        }

        Msg::GroupsFetched(Err(fetch_error)) => {
            log!("Error fetching groups {}", fetch_error);
        }

        Msg::PlayGame(input) => {
            let group = model.groups.get_mut(&input.group_id).unwrap();
            group.play_game(input.game_id, input.score);
        }
        _ => {}
    }
}

async fn get_teams() -> fetch::Result<Teams> {
    Request::new("http://192.168.0.15:8000/get_teams")
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

async fn get_groups() -> fetch::Result<Groups> {
    Request::new("http://192.168.0.15:8000/get_groups")
        .fetch()
        .await?
        .check_status()?
        .json()
        .await
}

fn view(model: &Model) -> Vec<Node<Msg>> {
    nodes![
        view_header(),
        view_group_play(&model.groups, &model.teams, &model.group_rules),
    ]
}

fn view_header() -> Node<Msg> {
    header![C!["header"], h1!["Group"],]
}

fn view_group_play<T: Tiebreaker>(groups: &Groups, teams: &Teams, rules: &Rules<T>) -> Node<Msg> {
    section![
        C!["group_play"],
        groups
            .iter()
            .map(|(group_id, group)| { view_group(group_id, group, teams, rules) })
    ]
}

fn view_group<T: Tiebreaker>(
    id: &GroupId,
    group: &Group,
    teams: &Teams,
    rules: &Rules<T>,
) -> Node<Msg> {
    div![
        //C![format!("group_{}", id).to_ascii_lowercase()],
        C!["group"],
        h2!(format!("{}", id)),
        format_group_table(group, teams, rules),
        format_group_games(id, group, teams),
    ]
}

fn format_group_table<T: Tiebreaker>(group: &Group, teams: &Teams, rules: &Rules<T>) -> Node<Msg> {
    let group_order = order_group(group, rules);
    let stats = DisplayTable::new(group, &group_order);
    stats.format(teams)
}

fn format_group_games(group_id: &GroupId, group: &Group, teams: &Teams) -> Node<Msg> {
    div![
        C!["games"],
        h3!("Games"),
        table![
            group.played_games().map(|game| { game.format(teams) }),
            group
                .unplayed_games()
                .map(|game| { game.format(&(teams, *group_id)) })
        ]
    ]
}

#[wasm_bindgen(start)]
pub fn start() {
    let root_element = document()
        .get_elements_by_class_name("tournament")
        .item(0)
        .expect("element with the class `tournament`");

    App::start(root_element, init, update, view);
}
