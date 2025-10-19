use crate::team::format_team_flag;
use leptos::prelude::*;
use std::convert::From;
use wwc_core::game::{GoalDiff, NumGames};
use wwc_core::group::{Group, stats::GameStat, stats::TableStats};
use wwc_core::group::{GroupPoint, TeamOrder};
use wwc_core::team::{Team, TeamId, Teams};

pub(crate) struct DisplayTable(Vec<(TeamId, DisplayTableRow)>);

impl DisplayTable {
    pub(crate) fn new(group: &Group, group_order: &TeamOrder) -> Self {
        let full_table = TableStats::team_stats(group);
        let tmp = group_order
            .iter()
            .map(|id| (*id, DisplayTableRow::from(*full_table.get(id).unwrap())))
            .collect();
        DisplayTable(tmp)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &(TeamId, DisplayTableRow)> {
        self.0.iter()
    }
}

#[component]
pub fn DisplayTableView(group: Group, teams: Teams, group_order: TeamOrder) -> impl IntoView {
    let display_table = DisplayTable::new(&group, &group_order);
    let rows: Vec<_> = display_table
        .iter()
        .map(|(team_id, stat)| {
            let team = teams
                .get(team_id)
                .unwrap_or_else(|| panic!("No team id: {}", team_id))
                .clone();
            (*team_id, stat.clone(), team)
        })
        .collect();

    view! {
        <div class="group-table">
            <table>
                <tr>
                    <th>""</th>
                    <th>""</th>
                    <th>"pl"</th>
                    <th>"+/-"</th>
                    <th>"p"</th>
                </tr>
                {rows
                    .into_iter()
                    .map(|(_, stat, team)| {
                        view! { <DisplayTableRowView stat=stat team=team/> }
                    })
                    .collect_view()}
            </table>
        </div>
    }
}

#[derive(Clone)]
pub(crate) struct DisplayTableRow {
    games_played: NumGames,
    points: GroupPoint,
    goal_diff: GoalDiff,
}

#[component]
fn DisplayTableRowView(stat: DisplayTableRow, team: Team) -> impl IntoView {
    let flag_class = format_team_flag(&team);

    view! {
        <tr>
            <td>{team.fifa_code.to_string()}</td>
            <td><span class={flag_class}></span></td>
            <td>{stat.games_played.to_string()}</td>
            <td>{stat.goal_diff.to_string()}</td>
            <td>{stat.points.to_string()}</td>
        </tr>
    }
}

impl From<TableStats> for DisplayTableRow {
    fn from(x: TableStats) -> Self {
        DisplayTableRow {
            games_played: x.games_played,
            points: x.points,
            goal_diff: x.goal_diff,
        }
    }
}
