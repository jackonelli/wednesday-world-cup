use crate::game::{GoalCount, GoalDiff};
use crate::group::game::PlayedGroupGame;
use crate::group::Group;
use crate::team::TeamId;
use derive_more::{Add, AddAssign, From};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops;

/// Primary stats for a group team
///
/// Collection of statistics that can be aggregated over games,
/// as opposed to stats based on a teams performance versus another specific team.
/// The name primary refers to the fact that it is (usually) the
/// first statistics that is used to determine group ordering.
/// TODO: Remove, DEPRECATED
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Add, AddAssign)]
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

impl PartialOrd for PrimaryStats {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrimaryStats {
    fn cmp(&self, other: &Self) -> Ordering {
        self.points
            .cmp(&other.points)
            .then(self.goal_diff.cmp(&other.goal_diff))
            .then(self.goals_scored.cmp(&other.goals_scored))
    }
}

impl num::Zero for PrimaryStats {
    fn zero() -> Self {
        PrimaryStats::default()
    }
    fn is_zero(&self) -> bool {
        self.points.0 == 0 && self.goal_diff.0 == 0 && self.goals_scored.0 == 0
    }
}

pub trait UnaryStat: Ord + Copy + num::Zero + ops::AddAssign {
    fn stat(game: &PlayedGroupGame) -> (Self, Self);

    fn team_stats(group: &Group) -> HashMap<TeamId, Self> {
        let team_map = group.teams().map(|team| (team, Self::zero())).collect();
        group.played_games.iter().fold(team_map, |mut acc, game| {
            let (delta_home_stat, delta_away_stat) = Self::stat(game);

            let stats = acc
                .get_mut(&game.home)
                .expect("TeamId will always be present");
            *stats += delta_home_stat;

            let stats = acc
                .get_mut(&game.away)
                .expect("TeamId will always be present");
            *stats += delta_away_stat;
            acc
        })
    }
}

#[derive(Default, Debug, Clone, Copy, From, Eq, PartialEq, Ord, PartialOrd, Add, AddAssign)]
pub struct GroupPoint(pub u8);

impl UnaryStat for GroupPoint {
    fn stat(game: &PlayedGroupGame) -> (Self, Self) {
        let score = &game.score;
        if score.home > score.away {
            (GroupPoint(3), GroupPoint(0))
        } else if score.home < score.away {
            (GroupPoint(0), GroupPoint(3))
        } else {
            (GroupPoint(1), GroupPoint(1))
        }
    }
}

impl num::Zero for GroupPoint {
    fn zero() -> GroupPoint {
        GroupPoint(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    #[test]
    fn primary_stats_points() {
        let stats_1 = PrimaryStats::new(0, 2, 4);
        let stats_2 = PrimaryStats::new(1, 0, 0);
        more_asserts::assert_gt!(stats_2, stats_1);
    }

    #[test]
    fn primary_stats_points_goaldiff() {
        let stats_1 = PrimaryStats::new(1, 2, 4);
        let stats_2 = PrimaryStats::new(1, 0, 0);
        more_asserts::assert_gt!(stats_1, stats_2);
    }

    #[test]
    fn primary_stats_points_goaldiff_goalsscored() {
        let stats_1 = PrimaryStats::new(3, -2, 4);
        let stats_2 = PrimaryStats::new(3, -2, 0);
        more_asserts::assert_gt!(stats_1, stats_2);
    }
    #[test]
    fn sort_in_vector() {
        let stats_1 = PrimaryStats::new(3, -2, 4);
        let stats_2 = PrimaryStats::new(3, -2, 0);
        let stats_3 = PrimaryStats::new(1, 2, 4);
        let stats_4 = PrimaryStats::new(1, 0, 0);
        let true_vec = vec![stats_1, stats_2, stats_3, stats_4];
        let mut stats_vec = true_vec.clone();
        stats_vec.shuffle(&mut thread_rng());
        stats_vec.sort();
        stats_vec.reverse();
        assert_eq!(true_vec, stats_vec);
    }
}
