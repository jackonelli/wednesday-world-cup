use crate::Msg;
use seed::{prelude::*, *};
use std::convert::From;
use wwc_core::game::{GoalDiff, NumGames};
use wwc_core::group::{stats::TableStats, stats::UnaryStat, Group};
use wwc_core::group::{GroupOrder, GroupPoint};
use wwc_core::team::{Team, TeamId};

pub struct DisplayTable(Vec<(TeamId, DisplayTableRow)>);

impl DisplayTable {
    pub(crate) fn iter(&self) -> impl Iterator<Item = &(TeamId, DisplayTableRow)> {
        self.0.iter()
    }
}

pub(crate) struct DisplayTableRow {
    games_played: NumGames,
    points: GroupPoint,
    goal_diff: GoalDiff,
}

impl DisplayTableRow {
    pub(crate) fn format(&self, team: &Team) -> Node<Msg> {
        p!(format!("{} {} {}", team.name, self.points, self.goal_diff))
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
