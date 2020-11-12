use crate::format::Format;
use crate::format_team_flag;
use crate::Msg;
use seed::{prelude::*, *};
use std::convert::From;
use wwc_core::game::{GoalDiff, NumGames};
use wwc_core::group::{stats::TableStats, stats::UnaryStat, Group};
use wwc_core::group::{GroupOrder, GroupPoint};
use wwc_core::team::{Team, TeamId, Teams};

pub struct DisplayTable(Vec<(TeamId, DisplayTableRow)>);

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

    pub(crate) fn iter(&self) -> impl Iterator<Item = &(TeamId, DisplayTableRow)> {
        self.0.iter()
    }
}

impl Format<'_> for DisplayTable {
    type Context = Teams;
    fn format(&self, cxt: &Teams) -> Node<Msg> {
        div![
            C!["group-table"],
            table![
                tr![th![""], th![""], th!["pl"], th!["+/-"], th!["p"]],
                self.iter().map(|(team_id, stat)| {
                    let team = cxt
                        .get(&team_id)
                        .unwrap_or_else(|| panic!("No team id: {}", team_id));
                    stat.format(team)
                })
            ]
        ]
    }
}

pub(crate) struct DisplayTableRow {
    games_played: NumGames,
    points: GroupPoint,
    goal_diff: GoalDiff,
}

impl Format<'_> for DisplayTableRow {
    type Context = Team;
    fn format(&self, cxt: &Team) -> Node<Msg> {
        tr![
            td![cxt.fifa_code.to_string()],
            td![format_team_flag(cxt)],
            td![self.games_played.to_string()],
            td![self.goal_diff.to_string()],
            td![self.points.to_string()]
        ]
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
