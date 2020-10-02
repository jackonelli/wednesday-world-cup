//! Group game
//!
//! Data and functionality related to games in the group stage of the tournament.
//! This group is less restricted than a `PlayoffGame` since the `Score` is freer,
//! e.g. draws are allowed and there are no additional penalty shoot-out score.
//!
//! The two game structs [`UnplayedGroupGame`](struct.UnPlayedGroupGame.html) and [`PlayedGroupGame`](struct.PlayedGroupGame.html)
//! are the fundamental datastructure for the group; all other properties and statistics are
//! derived from them.
use crate::fair_play::FairPlayScore;
use crate::game::GoalDiff;
use crate::game::{Game, GoalCount};
use crate::group::stats::UnaryStat;
use crate::group::GroupError;
use crate::group::GroupPoint;
use crate::team::TeamId;
use crate::Date;
use derive_more::{Add, AddAssign, Display, From};
use serde::{Deserialize, Serialize};

/// Score associated with [`PlayedGroupGame`](struct.PlayedGroupGame.html)
///
/// Determines the outcome of a game which can be, win, loss or draw.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Score {
    pub home: GoalCount,
    pub away: GoalCount,
}

impl<T: Into<GoalCount>> From<(T, T)> for Score {
    fn from(x: (T, T)) -> Self {
        let (home, away) = x;
        Self {
            home: home.into(),
            away: away.into(),
        }
    }
}

/// Not yet played group game
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UnplayedGroupGame {
    pub id: GroupGameId,
    pub home: TeamId,
    pub away: TeamId,
    pub date: Date,
}

impl UnplayedGroupGame {
    /// Fallible constructor.
    ///
    /// Fails if `TeamId`'s are not different for home and away team.
    pub fn try_new<G: Into<GroupGameId>, T: Into<TeamId>>(
        id: G,
        home: T,
        away: T,
        date: Date,
    ) -> Result<Self, GroupError> {
        let home = home.into();
        let away = away.into();
        if home != away {
            Ok(Self {
                id: id.into(),
                home,
                away,
                date,
            })
        } else {
            Err(GroupError::GameTeamsNotUnique)
        }
    }

    /// Transform unplayed game to played.
    ///
    /// Only (public) way of constructing a [`PlayedGroupGame`](struct.PlayedGroupGame.html).
    pub fn play(self, score: Score, fair_play: FairPlayScore) -> PlayedGroupGame {
        PlayedGroupGame {
            id: self.id,
            home: self.home,
            away: self.away,
            date: self.date,
            score,
            fair_play,
        }
    }
}

impl Game for UnplayedGroupGame {
    fn home_team(&self) -> TeamId {
        self.home
    }
    fn away_team(&self) -> TeamId {
        self.away
    }
}

/// Played group game
///
/// Can only be constructed by invoking the [`.play`](struct.UnplayedGroupGame.html#method.play) method on a
/// [`UnplayedGroupGame`](struct.UnplayedGroupGame.html)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PlayedGroupGame {
    pub id: GroupGameId,
    pub home: TeamId,
    pub away: TeamId,
    pub score: Score,
    pub fair_play: FairPlayScore,
    date: Date,
}

impl PlayedGroupGame {
    /// Points awarded to (home, away) teams respectively.
    pub fn points(&self) -> (GroupPoint, GroupPoint) {
        GroupPoint::stat(self)
    }

    /// Goal difference for (home, away) teams respectively.
    pub fn goal_diff(&self) -> (GoalDiff, GoalDiff) {
        GoalDiff::stat(self)
    }

    /// Goals scored for (home, away) teams respectively.
    pub fn goals_scored(&self) -> (GoalCount, GoalCount) {
        GoalCount::stat(self)
    }

    /// Fallible constructor for a played group game
    ///
    /// Used in-crate only for easier test setup.
    #[cfg(test)]
    pub(crate) fn try_new<
        G: Into<GroupGameId>,
        T: Into<TeamId>,
        S: Into<Score>,
        F: Into<FairPlayScore>,
    >(
        id: G,
        home: T,
        away: T,
        score: S,
        fair_play: F,
        date: Date,
    ) -> Result<Self, GroupError> {
        let home = home.into();
        let away = away.into();
        if home != away {
            Ok(Self {
                id: id.into(),
                home,
                away,
                score: score.into(),
                fair_play: fair_play.into(),
                date,
            })
        } else {
            Err(GroupError::GameTeamsNotUnique)
        }
    }
}

impl Game for PlayedGroupGame {
    fn home_team(&self) -> TeamId {
        self.home
    }
    fn away_team(&self) -> TeamId {
        self.away
    }
}

#[derive(
    Debug,
    Display,
    Deserialize,
    Serialize,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
    Add,
    AddAssign,
    From,
)]
pub struct GroupGameId(pub u8);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn home_win() {
        let game =
            PlayedGroupGame::try_new(0, 0, 1, (3, 0), FairPlayScore::default(), Date::mock())
                .unwrap();
        let (home, away) = game.points();
        assert_eq!(home, GroupPoint(3));
        assert_eq!(away, GroupPoint(0));
    }

    #[test]
    fn away_win() {
        let game =
            PlayedGroupGame::try_new(0, 0, 1, (0, 2), FairPlayScore::default(), Date::mock())
                .unwrap();
        let (home, away) = game.points();
        assert_eq!(home, GroupPoint(0));
        assert_eq!(away, GroupPoint(3));
    }

    #[test]
    fn draw() {
        let game =
            PlayedGroupGame::try_new(0, 0, 1, (0, 0), FairPlayScore::default(), Date::mock())
                .unwrap();
        let (home, away) = game.points();
        assert_eq!(home, GroupPoint(1));
        assert_eq!(away, GroupPoint(1));
    }
}
