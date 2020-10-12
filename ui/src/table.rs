use crate::Msg;
use seed::{prelude::*, *};
use std::convert::From;
use wwc_core::game::{GoalDiff, NumGames};
use wwc_core::group::{stats::TableStats, stats::UnaryStat, Group};
use wwc_core::group::{GroupOrder, GroupPoint};
use wwc_core::team::{Team, TeamId, Teams};

pub struct DisplayTable(Vec<(TeamId, DisplayTableRow)>);

impl DisplayTable {
    pub(crate) fn iter(&self) -> impl Iterator<Item = &(TeamId, DisplayTableRow)> {
        self.0.iter()
    }

    pub(crate) fn format(&self, teams: &Teams) -> Node<Msg> {
        table![
            tr![th![""], th![""], th!["pl"], th!["+/-"], th!["p"]],
            self.iter().map(|(team_id, stat)| {
                let team = teams.get(&team_id).unwrap();
                stat.format(team)
            })
        ]
    }
}

impl DisplayTable {
    pub fn new(group: &Group, group_order: &GroupOrder) -> Self {
        let full_table = TableStats::team_stats(group);
        let tmp = group_order
            .iter()
            .map(|id| {
                (
                    *id,
                    DisplayTableRow::from(full_table.get(id).unwrap().clone()),
                )
            })
            .collect();
        DisplayTable(tmp)
    }
}

pub(crate) struct DisplayTableRow {
    games_played: NumGames,
    points: GroupPoint,
    goal_diff: GoalDiff,
}

impl DisplayTableRow {
    pub(crate) fn format(&self, team: &Team) -> Node<Msg> {
        tr![
            td![team.fifa_code.to_string()],
            td![format_team_flag(team)],
            td![self.games_played.to_string()],
            td![self.goal_diff.to_string()],
            td![self.points.to_string()]
        ]
    }
}

fn format_team_flag(team: &Team) -> Node<Msg> {
    span![C![format!(
        "tournament-group__flag flag-icon flag-icon-{}",
        team.iso2
    )]]
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
