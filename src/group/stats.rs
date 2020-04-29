use crate::fair_play::FairPlayScore;
use crate::game::{GoalCount, GoalDiff, NumGames};
use crate::team::TeamId;
use derive_more::{Add, AddAssign, From};
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Default, Eq, PartialEq)]
pub struct PrimaryStats {
    points: GroupPoint,
    goal_diff: GoalDiff,
    goals_scored: GoalCount,
}

impl PrimaryStats {
    pub(crate) fn new<P: Into<GroupPoint>, D: Into<GoalDiff>, G: Into<GoalCount>>(
        points: P,
        goal_diff: D,
        goals_scored: G,
    ) -> Self {
        PrimaryStats {
            points: points.into(),
            goal_diff: goal_diff.into(),
            goals_scored: goals_scored.into(),
        }
    }
}

impl std::cmp::PartialOrd for PrimaryStats {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for PrimaryStats {
    fn cmp(&self, other: &Self) -> Ordering {
        self.points
            .cmp(&other.points)
            .then(self.goal_diff.cmp(&other.goal_diff))
            .then(self.goals_scored.cmp(&other.goals_scored))
    }
}

impl std::ops::AddAssign for PrimaryStats {
    fn add_assign(&mut self, rhs: PrimaryStats) {
        self.points += rhs.points;
        self.goal_diff += rhs.goal_diff;
        self.goals_scored += rhs.goals_scored;
    }
}

#[derive(Default, Debug, Clone, Copy, From, Eq, PartialEq, Ord, PartialOrd, Add, AddAssign)]
pub struct GroupPoint(pub u8);

impl Unary for GroupPoint {}

impl num::Zero for GroupPoint {
    fn zero() -> GroupPoint {
        GroupPoint(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

pub trait Unary {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_stats_points() {
        let stats_1 = PrimaryStats::new(0, 2, 4);
        let stats_2 = PrimaryStats::new(1, 0, 0);
        more_asserts::assert_lt!(stats_1, stats_2);
    }
}
