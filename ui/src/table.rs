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

    pub(crate) fn gen_table(&self) -> Node<Msg> {
        table![
            tr![th!["Team"], th!["Played games"], th!["+/-"], th!["p"]],
            self.iter().map(|(team_id, stat)| {
                tr![
                    td![team_id],
                    td![stat.games_played],
                    td![stat.goal_diff],
                    td![stat.points]
                ]
            })
        ]
    }

    pub fn gen_ul(&self) -> Node<Msg> {
        ul![self.iter().map(|(team_id, stat)| {
            li![format!(
                "{}, {}, {}, {}",
                team_id, stat.games_played, stat.goal_diff, stat.points
            )]
        })]
    }

    pub(crate) fn format(&self, teams: &Teams) -> Node<Msg> {
        table![
            tr![th!["Team"], th!["Played games"], th!["+/-"], th!["p"]],
            self.iter().map(|(team_id, stat)| {
                let team = teams.get(&team_id).unwrap();
                tr![
                    td![team.name],
                    td![stat.games_played],
                    td![stat.goal_diff],
                    td![stat.points]
                ]
            })
        ]
        //ul![self.iter().map(|(team_id, stat)| {
        //    let team = teams.get(&team_id).unwrap();
        //    li![C!["group-team"], el_key(&team_id), stat.format(&team)]
        //})]
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
            td![team.name],
            td![self.games_played],
            td![self.goal_diff],
            td![self.points]
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
