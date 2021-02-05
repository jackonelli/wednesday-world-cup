//! Group statistics
use crate::fair_play::{FairPlayValue, FifaFairPlayValue};
use crate::game::{GoalCount, GoalDiff, NumGames};
use crate::group::game::PlayedGroupGame;
use crate::group::{Group, GroupPoint};
use crate::team::TeamId;
use derive_more::{Add, AddAssign};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops;

// TODO: This could perhaps be more efficient with the cool group by
// https://docs.rs/itertools/0.10.0/itertools/trait.Itertools.html#method.into_grouping_map
// i.e. don't fold but map all games to (team_id, stat), then eff. fold to team stats map.
/// Statistic calculated from a single game.
///
/// Implentor needs to provide the actual [`UnaryStat::stat`] function,
/// which calculates the statistics (for both teams) for a single game.
/// The trait then provides default methods to calculate the statistic on group level.
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
        let team_map = group.team_ids().map(|team| (team, Self::zero())).collect();
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
/// Internal helper function for the [`UnaryStat`] trait.
///
/// # Panics
///
/// Unwrap's do not panic if [`TeamId`]'s of `game.home` and `game.away` are members of `acc`:
/// - Calling this from [`UnaryStat::team_stats`], [`TeamId`]'s will always be present, checked in [Group] constructor.
/// - Calling this from [`UnaryStat::internal_team_stats`] is ok since the unwrap's would panic iff `acc` would
///   not contain `game.home` or `game.away`, which is exactly the predicate that the
///   `group.played_games` are filtered by.
/// - Other calls do not exist (private fn), when adding a call: Take care to uphold this invariant!
fn calc_and_assign_stat<T: UnaryStat>(
    acc: HashMap<TeamId, T>,
    game: &PlayedGroupGame,
) -> HashMap<TeamId, T> {
    let mut acc = acc;
    let (delta_home_stat, delta_away_stat) = T::stat(game);

    let stats = acc.get_mut(&game.home).unwrap();
    *stats += delta_home_stat;

    let stats = acc.get_mut(&game.away).unwrap();
    *stats += delta_away_stat;
    acc
}

impl UnaryStat for GroupPoint {
    /// Group points from played game.
    ///
    /// ```
    /// # use wwc_core::group::stats::UnaryStat;
    /// # use wwc_core::group::game::{UnplayedGroupGame};
    /// # use wwc_core::game::{Score, GameId};
    /// # use wwc_core::group::GroupPoint;
    /// # use wwc_core::team::TeamId;
    /// # use wwc_core::Date;
    /// # use wwc_core::fair_play::{FairPlay, FairPlayScore};
    /// let score = Score::from((1, 0));
    /// let fair_play_score = FairPlayScore::new(FairPlay::new(1, 0, 0, 0), FairPlay::new(0, 0, 0, 0));
    /// let game = UnplayedGroupGame::try_new(GameId::from(0), TeamId::from(1), TeamId::from(2), Date::mock())
    ///     .unwrap()
    ///     .play(score, fair_play_score);
    /// let (home, away) = GroupPoint::stat(&game);
    /// assert_eq!(home, GroupPoint(3));
    /// assert_eq!(away, GroupPoint(0));
    /// ```
    fn stat(game: &PlayedGroupGame) -> (Self, Self) {
        let score = &game.score;
        match score.home.cmp(&score.away) {
            Ordering::Greater => (GroupPoint(3), GroupPoint(0)),
            Ordering::Less => (GroupPoint(0), GroupPoint(3)),
            Ordering::Equal => (GroupPoint(1), GroupPoint(1)),
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

impl<T> UnaryStat for T
where
    T: FairPlayValue + num::Zero + ops::AddAssign,
{
    fn stat(game: &PlayedGroupGame) -> (T, T) {
        (
            T::from_fair_play(&game.fair_play.home),
            T::from_fair_play(&game.fair_play.away),
        )
    }
}

#[derive(Add, AddAssign, Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Copy)]
pub struct NumWins(NumGames);

impl UnaryStat for NumWins {
    fn stat(game: &PlayedGroupGame) -> (Self, Self) {
        let (points_home, points_away) = GroupPoint::stat(game);
        let wins_home = NumWins(NumGames((points_home == GroupPoint(3)) as u32));
        let wins_away = NumWins(NumGames((points_away == GroupPoint(3)) as u32));
        (wins_home, wins_away)
    }
}

impl num::Zero for NumWins {
    fn zero() -> Self {
        NumWins(NumGames::zero())
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

///Convenience struct for combining all common stats
///
///Does not impl UnaryStat even though it could in principle do it.
///Defining an order (impl Ord) defeats the purpose of composing rules.
#[derive(Add, AddAssign, Debug, Clone, Eq, PartialEq)]
pub struct TableStats {
    pub points: GroupPoint,
    pub goal_diff: GoalDiff,
    pub goals_scored: GoalCount,
    pub goals_conceded: GoalCount,
    pub fair_play_score: FifaFairPlayValue,
    pub games_played: NumGames,
    pub wins: NumGames,
    pub losses: NumGames,
    pub draws: NumGames,
}

impl TableStats {
    fn new<GP, GC, FFP, NG>(
        points: GP,
        goals_scored: GC,
        goals_conceded: GC,
        fair_play_score: FFP,
        wins: NG,
        losses: NG,
        draws: NG,
    ) -> Self
    where
        GP: Into<GroupPoint>,
        GC: Into<GoalCount> + Copy,
        FFP: Into<FifaFairPlayValue>,
        NG: Into<NumGames> + Copy,
    {
        Self {
            points: points.into(),
            goal_diff: goals_scored.into() - goals_conceded.into(),
            goals_scored: goals_scored.into(),
            goals_conceded: goals_conceded.into(),
            fair_play_score: fair_play_score.into(),
            games_played: wins.into() + losses.into() + draws.into(),
            wins: wins.into(),
            losses: losses.into(),
            draws: draws.into(),
        }
    }
}

impl UnaryStat for TableStats {
    fn stat(game: &PlayedGroupGame) -> (Self, Self) {
        let (points_home, points_away) = GroupPoint::stat(game);
        let (goals_scored_home, goals_scored_away) = GoalCount::stat(game);
        let (fair_play_home, fair_play_away) = FifaFairPlayValue::stat(game);
        let (wins_home, wins_away) = NumWins::stat(game);
        let losses_home = NumGames((points_home == GroupPoint(0)) as u32);
        let draws_home = NumGames((points_home == GroupPoint(1)) as u32);
        let losses_away = NumGames((points_away == GroupPoint(0)) as u32);
        let draws_away = NumGames((points_away == GroupPoint(1)) as u32);
        let home = TableStats::new(
            points_home,
            goals_scored_home,
            goals_scored_away,
            fair_play_home,
            wins_home.0,
            losses_home,
            draws_home,
        );
        let away = TableStats::new(
            points_away,
            goals_scored_away,
            goals_scored_home,
            fair_play_away,
            wins_away.0,
            losses_away,
            draws_away,
        );
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
            fair_play_score: FifaFairPlayValue::zero(),
            games_played: NumGames::zero(),
            wins: NumGames::zero(),
            losses: NumGames::zero(),
            draws: NumGames::zero(),
        }
    }

    fn is_zero(&self) -> bool {
        self.points.is_zero()
            && self.goal_diff.is_zero()
            && self.goals_scored.is_zero()
            && self.goals_conceded.is_zero()
            && self.fair_play_score.is_zero()
            && self.games_played.is_zero()
            && self.wins.is_zero()
            && self.draws.is_zero()
            && self.losses.is_zero()
    }
}

impl std::fmt::Display for TableStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\t{}\t{}\t{}",
            self.points, self.goal_diff, self.goals_scored, self.fair_play_score,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group::tests::mock_data;
    use crate::group::GroupId;
    use num::Zero;

    #[test]
    fn mock_teams_stats() {
        let (groups, _) = mock_data();
        let group_a = groups.get(&GroupId::from('A')).unwrap();
        let mut truth = HashMap::new();
        truth.insert(TeamId::from(1), TableStats::new(3, 2, 1, 0, 1, 0, 0));
        truth.insert(TeamId::from(2), TableStats::new(0, 1, 2, 0, 0, 1, 0));
        truth.insert(TeamId::from(3), TableStats::zero());
        truth.insert(TeamId::from(4), TableStats::zero());
        assert_eq!(truth, TableStats::team_stats(group_a));
    }
}
