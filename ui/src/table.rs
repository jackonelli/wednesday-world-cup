use crate::app::Msg;
use crate::format::Format;
use crate::team::format_team_flag;
use seed::{prelude::*, *};
use std::convert::From;
use wwc_core::game::{GoalDiff, NumGames};
use wwc_core::group::{stats::GameStat, stats::TableStats, Group};
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

impl Format<'_> for DisplayTable {
    type Context = Teams;
    fn format(&self, ctx: &Teams) -> Node<Msg> {
        div![
            C!["group-table"],
            table![
                tr![th![""], th![""], th!["pl"], th!["+/-"], th!["p"]],
                self.iter().map(|(team_id, stat)| {
                    let team = ctx
                        .get(team_id)
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
    fn format(&self, ctx: &Team) -> Node<Msg> {
        tr![
            td![ctx.fifa_code.to_string()],
            td![format_team_flag(ctx)],
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
