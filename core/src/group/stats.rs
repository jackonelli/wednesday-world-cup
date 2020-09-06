use crate::fair_play::FairPlayValue;
use crate::game::{GoalCount, GoalDiff, NumGames};
use crate::group::game::PlayedGroupGame;
use crate::group::{Group, GroupPoint};
use crate::team::TeamId;
use derive_more::{Add, AddAssign};
use std::collections::HashMap;
use std::ops;

fn team_stats<T: num::Zero + ops::AddAssign>(
    group: &Group,
    stat: fn(&PlayedGroupGame) -> (T, T),
) -> HashMap<TeamId, T> {
    let team_map = group.teams().map(|team| (team, T::zero())).collect();
    group.played_games.iter().fold(team_map, |mut acc, game| {
        let (delta_home_stat, delta_away_stat) = stat(game);

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

pub trait UnaryStat: Ord + Copy + num::Zero + ops::AddAssign {
    fn stat(game: &PlayedGroupGame) -> (Self, Self);

    fn team_stats(group: &Group) -> HashMap<TeamId, Self> {
        team_stats(group, Self::stat)
    }
}

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

impl UnaryStat for GoalDiff {
    fn stat(game: &PlayedGroupGame) -> (Self, Self) {
        let goal_diff = game.score.home - game.score.away;
        (goal_diff, -goal_diff)
    }
}

impl UnaryStat for GoalCount {
    fn stat(game: &PlayedGroupGame) -> (Self, Self) {
        (game.score.home, game.score.away)
    }
}

impl UnaryStat for FairPlayValue {
    fn stat(game: &PlayedGroupGame) -> (Self, Self) {
        (game.fair_play.home, game.fair_play.away)
    }
}

///Convenience struct for combining all common stats
///
///Does not impl UnaryStat even though it could in principle do it.
///Defining an order (impl Ord) defeats the purpose of composing rules.
#[derive(Add, AddAssign, Debug, Clone)]
pub struct TableStats {
    pub points: GroupPoint,
    pub goal_diff: GoalDiff,
    pub goals_scored: GoalCount,
    pub goals_conceded: GoalCount,
    pub games_played: NumGames,
    pub wins: NumGames,
    pub losses: NumGames,
    pub draws: NumGames,
}

impl TableStats {
    pub fn team_stats(group: &Group) -> HashMap<TeamId, Self> {
        team_stats(group, Self::stat)
    }

    fn stat(game: &PlayedGroupGame) -> (Self, Self) {
        let (points_home, points_away) = GroupPoint::stat(game);
        let (goal_diff_away, goal_diff_home) = GoalDiff::stat(game);
        let (goals_scored_home, goals_scored_away) = GoalCount::stat(game);
        let wins_home = NumGames((points_home == GroupPoint(3)) as u8);
        let losses_home = NumGames((points_home == GroupPoint(0)) as u8);
        let draws_home = NumGames((points_home == GroupPoint(1)) as u8);
        let wins_away = NumGames((points_away == GroupPoint(3)) as u8);
        let losses_away = NumGames((points_away == GroupPoint(0)) as u8);
        let draws_away = NumGames((points_away == GroupPoint(1)) as u8);
        let home = TableStats {
            points: points_home,
            goal_diff: goal_diff_home,
            goals_scored: goals_scored_home,
            goals_conceded: goals_scored_away,
            games_played: NumGames(1),
            wins: wins_home,
            losses: losses_home,
            draws: draws_home,
        };
        let away = TableStats {
            points: points_away,
            goal_diff: goal_diff_away,
            goals_scored: goals_scored_away,
            goals_conceded: goals_scored_home,
            games_played: NumGames(1),
            wins: wins_away,
            losses: losses_away,
            draws: draws_away,
        };
        (home, away)
    }
}

impl num::Zero for TableStats {
    fn zero() -> Self {
        TableStats {
            points: GroupPoint::zero(),
            goal_diff: GoalDiff::zero(),
            goals_scored: GoalCount::zero(),
            goals_conceded: GoalCount::zero(),
            games_played: NumGames::zero(),
            wins: NumGames::zero(),
            losses: NumGames::zero(),
            draws: NumGames::zero(),
        }
    }

    fn is_zero(&self) -> bool {
        return self.points.is_zero()
            && self.goal_diff.is_zero()
            && self.goals_scored.is_zero()
            && self.goals_conceded.is_zero()
            && self.games_played.is_zero()
            && self.wins.is_zero()
            && self.draws.is_zero()
            && self.losses.is_zero();
    }
}

#[cfg(test)]
mod tests {
    // TODO: Test TableStats
}
