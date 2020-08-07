#![allow(clippy::wildcard_imports)]
#![allow(dead_code, unused_variables)]
use seed::{prelude::*, *};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::mem;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use ulid::Ulid;
use wwc_core::{
    group::{mock_data, Group},
    team::{Team, TeamId},
};
const ENTER_KEY: &str = "Enter";
const ESCAPE_KEY: &str = "Escape";

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    let (mock_groups, mock_teams) = mock_data();
    Model {
        groups: mock_groups,
        teams: mock_teams,
        base_url: Url::new(),
    }
}

struct Model {
    groups: Vec<Group>,
    teams: HashMap<TeamId, Team>,
    base_url: Url,
}

enum Msg {
    UrlChanged(subs::UrlChanged),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {}

fn view(model: &Model) -> Vec<Node<Msg>> {
    nodes![view_header(), view_group_play(&model.groups, &model.teams),]
}

fn view_header() -> Node<Msg> {
    header![C!["header"], h1!["Group"],]
}

fn view_group_play(groups: &[Group], teams: &HashMap<TeamId, Team>) -> Node<Msg> {
    section![
        C!["main"],
        groups.iter().map(|group| { view_group(group, teams) })
    ]
}

fn view_group(group: &Group, teams: &HashMap<TeamId, Team>) -> Node<Msg> {
    ul![
        C!["group"],
        group.teams().map(|team_id| {
            li![
                C!["group-team"],
                el_key(&team_id),
                div![format!("Team {}", team_id)],
            ]
        })
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
