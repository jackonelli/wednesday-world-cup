use crate::fair_play::FairPlayScore;
use crate::game::{GoalCount, GoalDiff, NumGames};
use crate::team::TeamId;
use derive_more::{Add, AddAssign, From};
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct GroupStats(pub HashMap<TeamId, GroupTeamStats>);

impl GroupStats {
    pub fn get(&self, id: &TeamId) -> Option<&GroupTeamStats> {
        self.0.get(id)
    }
    pub fn rank_teams() -> Vec<TeamId> {
        todo!()
    }
}

#[derive(Debug, Default, Eq, PartialEq, AddAssign)]
pub struct GroupTeamStats {
    points: GroupPoint,
    games_played: NumGames,
    goals_scored: GoalCount,
    goals_conceded: GoalCount,
    fair_play: FairPlayScore,
}

impl GroupTeamStats {
    pub(crate) fn new<
        P: Into<GroupPoint>,
        N: Into<NumGames>,
        G: Into<GoalCount>,
        F: Into<FairPlayScore>,
    >(
        points: P,
        games_played: N,
        goals_scored: G,
        goals_conceded: G,
        fair_play: F,
    ) -> Self {
        GroupTeamStats {
            points: points.into(),
            games_played: games_played.into(),
            goals_scored: goals_scored.into(),
            goals_conceded: goals_conceded.into(),
            fair_play: fair_play.into(),
        }
    }

    pub fn goal_diff(&self) -> GoalDiff {
        self.goals_scored - self.goals_conceded
    }
}

impl std::cmp::PartialOrd for GroupTeamStats {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for GroupTeamStats {
    fn cmp(&self, other: &Self) -> Ordering {
        let pot_order = self.points.cmp(&other.points);
        if pot_order != Ordering::Equal {
            return pot_order;
        }
        let pot_order = self.goal_diff().cmp(&other.goal_diff());
        if pot_order != Ordering::Equal {
            return pot_order;
        }
        let pot_order = self.goals_scored.cmp(&other.goals_scored);
        if pot_order != Ordering::Equal {
            return pot_order;
        }
        let pot_order = self.goals_scored.cmp(&other.goals_scored);
        pot_order
    }
}

#[derive(Default, Debug, Clone, Copy, From, Eq, PartialEq, Ord, PartialOrd, Add, AddAssign)]
pub struct GroupPoint(pub u8);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn group_team_stats_ordering_points_and_games_played() {
        let stats_1 = GroupTeamStats::new(1, 2, 4, 2, 0);
        let stats_2 = GroupTeamStats::new(1, 0, 0, 1, 0);
        more_asserts::assert_gt!(stats_1, stats_2);
    }

    #[test]
    fn group_team_stats_ordering_points() {
        let stats_1 = GroupTeamStats::new(0, 2, 4, 2, 0);
        let stats_2 = GroupTeamStats::new(1, 0, 0, 1, 0);
        more_asserts::assert_lt!(stats_1, stats_2);
    }
}
