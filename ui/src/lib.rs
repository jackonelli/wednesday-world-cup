#![allow(clippy::wildcard_imports)]
#![allow(dead_code, unused_variables)]
use seed::{prelude::*, *};
use std::collections::HashMap;
use wwc_core::{
    group::{
        mock_data,
        order::{fifa_2018, order_group, Random, Rules, Tiebreaker},
        Group, GroupId, Groups,
    },
    team::{Team, TeamId},
};

mod table;
use table::DisplayTable;
const ENTER_KEY: &str = "Enter";
const ESCAPE_KEY: &str = "Escape";

type Teams = HashMap<TeamId, Team>;

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    let (mock_groups, mock_teams) = mock_data();
    Model {
        groups: mock_groups,
        teams: mock_teams,
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
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {}

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
        format_group_games(group, teams),
    ]
}

fn format_group_table<T: Tiebreaker>(group: &Group, teams: &Teams, rules: &Rules<T>) -> Node<Msg> {
    let group_order = order_group(group, rules);
    let stats = DisplayTable::new(group, &group_order);
    div![stats.format(teams)]
}

fn format_group_games(group: &Group, teams: &Teams) -> Node<Msg> {
    div![
        C!["games"],
        h3!("Games"),
        ul![group.played_games().map(|game| {
            li![
                C!["played_game"],
                el_key(&game.id),
                format!(
                    "{} {} - {} {}",
                    teams.get(&game.home).unwrap().name,
                    game.score.home,
                    game.score.away,
                    teams.get(&game.away).unwrap().name
                )
            ]
        })],
        ul![group.upcoming_games().map(|game| {
            li![
                C!["upcoming_game"],
                el_key(&game.id),
                format!(
                    "{}   -   {}",
                    teams.get(&game.home).unwrap().name,
                    teams.get(&game.away).unwrap().name
                )
            ]
        })]
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
