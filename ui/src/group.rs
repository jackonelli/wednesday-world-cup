use crate::app::Msg;
use crate::format::Format;
use crate::table::DisplayTable;
use seed::{prelude::*, *};
use wwc_core::{
    group::{
        order::{order_group, Rules, Tiebreaker},
        Group, GroupId, Groups,
    },
    team::Teams,
};
pub(crate) fn view_group_play<T: Tiebreaker>(
    groups: &Groups,
    teams: &Teams,
    rules: &Rules<T>,
) -> Node<Msg> {
    section![
        C!["group_play"],
        h2!["Groups"],
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
        h3!(format!("{}", id)),
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
        h4!("Games"),
        table![
            group
                .played_games()
                .map(|game| { game.format(&(teams, *group_id)) }),
            group
                .unplayed_games()
                .map(|game| { game.format(&(teams, *group_id)) })
        ]
    ]
}
