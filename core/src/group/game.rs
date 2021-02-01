//! Group game
//!
//! Data and functionality related to games in the group stage of the tournament.
//! This group is less restricted than a [`PlayoffGame`] since the [`Score`] is freer,
//! e.g. draws are allowed and there are no additional penalty shoot-out score.
//!
//! The two game structs [`UnplayedGroupGame`] and [`PlayedGroupGame`]
//! are the fundamental datastructure for the group; all other properties and statistics are
//! derived from them.
use crate::fair_play::FairPlayScore;
use crate::game::GameId;
use crate::game::GoalDiff;
use crate::game::Score;
use crate::game::{Game, GoalCount};
use crate::group::stats::UnaryStat;
use crate::group::GroupError;
use crate::group::GroupPoint;
use crate::team::TeamId;
use crate::Date;
use serde::{Deserialize, Serialize};

/// Not yet played group game
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UnplayedGroupGame {
    pub id: GameId,
    pub home: TeamId,
    pub away: TeamId,
    pub date: Date,
}

impl UnplayedGroupGame {
    /// Fallible constructor.
    ///
    /// # Errors
    ///
    /// Enforces distinct team id's
    pub fn try_new<G: Into<GameId>, T: Into<TeamId>>(
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
    /// Only (public) way of constructing a [`PlayedGroupGame`].
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
/// Can only be constructed by invoking the [`.play`] method on a
/// [`UnplayedGroupGame`]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PlayedGroupGame {
    pub id: GameId,
    pub home: TeamId,
    pub away: TeamId,
    pub score: Score,
    pub fair_play: FairPlayScore,
    pub(crate) date: Date,
}

impl PlayedGroupGame {
    /// Fallible constructor for a played group game
    ///
    /// # Errors
    ///
    /// Enforces distinct team id's
    pub fn try_new<G: Into<GameId>, T: Into<TeamId>, S: Into<Score>, F: Into<FairPlayScore>>(
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

    /// Transform played game to unplayed.
    pub fn unplay(self) -> UnplayedGroupGame {
        UnplayedGroupGame {
            id: self.id,
            home: self.home,
            away: self.away,
            date: self.date,
        }
    }

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
}

impl Game for PlayedGroupGame {
    fn home_team(&self) -> TeamId {
        self.home
    }
    fn away_team(&self) -> TeamId {
        self.away
    }
}

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
