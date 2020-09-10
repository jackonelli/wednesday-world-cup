//! Group statistics
use crate::fair_play::FairPlayValue;
use crate::game::{GoalCount, GoalDiff, NumGames};
use crate::group::game::PlayedGroupGame;
use crate::group::{Group, GroupPoint};
use crate::team::TeamId;
use derive_more::{Add, AddAssign};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops;

/// Statistic calculated from a single game.
///
/// Implentor needs to provide the actual `stat` function,
/// the trait provides default methods to calculate the statistic on group level.
pub trait UnaryStat: num::Zero + ops::AddAssign {
    /// Calculate statistic for a game.
    ///
    /// Unary statistics are necessarily symmetric: the game is the basis for the statistic for
    /// both home and away team.
    /// The tuple returned is (Home team stat, Away team stat).
    fn stat(game: &PlayedGroupGame) -> (Self, Self);

    /// Calculate statistic for all played games in a group.
    ///
    /// Statistics for all games are summed up and stored in a map of the teams.
    fn team_stats(group: &Group) -> HashMap<TeamId, Self> {
        let team_map = group.teams().map(|team| (team, Self::zero())).collect();
        group
            .played_games
            .iter()
            .fold(team_map, |acc, game| calc_and_assign_stat(acc, game))
    }

    /// Calculate statistic for filtered played games in a group.
    ///
    /// Only games where both home and away teams are members of the `team_filter` set.
    /// Statistics for the games are summed up and stored in a map of the teams.
    fn internal_team_stats(group: &Group, team_filter: &HashSet<&TeamId>) -> HashMap<TeamId, Self> {
        let team_map = team_filter
            .iter()
            .map(|team| (**team, Self::zero()))
            .collect();
        group
            .played_games
            .iter()
            .filter(|game| team_filter.contains(&game.home) && team_filter.contains(&game.away))
            .fold(team_map, |acc, game| calc_and_assign_stat(acc, game))
    }
}

/// Calculate stat for a game and assign to team map.
///
/// Unwrap's does not panic if TeamId's of `game.home` and `game.away` are members of `acc`:
/// - Calling this from [`team_stats`](trait.UnaryStat.html#method.team_stats), TeamId's will always be present, checked in Group constructor.
/// - Calling this from [`internal_team_stats`](trait.UnaryStat.html#method.internal_team_stats) is ok since the unwrap's would panic iff `acc` would
///   not contain `game.home` or `game.away`, which is exactly the predicate that the
///   `group.played_games` is filtered by.
/// - Other calls do not exist (private fn), when adding a call: Take care to uphold this invariant!
fn calc_and_assign_stat<T: UnaryStat>(
    mut acc: HashMap<TeamId, T>,
    game: &PlayedGroupGame,
) -> HashMap<TeamId, T> {
    let (delta_home_stat, delta_away_stat) = T::stat(game);

    let stats = acc.get_mut(&game.home).unwrap();
    *stats += delta_home_stat;

    let stats = acc.get_mut(&game.away).unwrap();
    *stats += delta_away_stat;
    acc
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

impl UnaryStat for TableStats {
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

impl std::fmt::Display for TableStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\t{}\t{}",
            self.points, self.goal_diff, self.goals_scored
        )
    }
}

#[cfg(test)]
mod tests {
    // TODO: Test TableStats
    // TODO: Test TeamStats
}
