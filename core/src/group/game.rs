//! Group game
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Score {
    pub home: GoalCount,
    pub away: GoalCount,
}

impl<T: Into<GoalCount>> From<(T, T)> for Score {
    fn from(x: (T, T)) -> Self {
        Self {
            home: x.0.into(),
            away: x.1.into(),
        }
    }
}

impl Score {
    pub fn new<T: Into<GoalCount>>(home: T, away: T) -> Self {
        Score {
            home: home.into(),
            away: away.into(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PreGroupGame {
    pub id: GroupGameId,
    pub home: TeamId,
    pub away: TeamId,
    pub date: Date,
}

impl PreGroupGame {
    pub fn try_new<G: Into<GroupGameId>, T: Into<TeamId> + Eq>(
        id: G,
        home: T,
        away: T,
        date: Date,
    ) -> Result<Self, GroupError> {
        if home != away {
            Ok(Self {
                id: id.into(),
                home: home.into(),
                away: away.into(),
                date,
            })
        } else {
            Err(GroupError::GameTeamsNotUnique)
        }
    }
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

impl Game for PreGroupGame {
    fn home_team(&self) -> TeamId {
        self.home
    }
    fn away_team(&self) -> TeamId {
        self.away
    }
}

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
    pub(crate) fn try_new<
        G: Into<GroupGameId>,
        T: Into<TeamId> + Eq,
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
        if home != away {
            Ok(Self {
                id: id.into(),
                home: home.into(),
                away: away.into(),
                score: score.into(),
                fair_play: fair_play.into(),
                date,
            })
        } else {
            Err(GroupError::GameTeamsNotUnique)
        }
    }
    pub fn points(&self) -> (GroupPoint, GroupPoint) {
        GroupPoint::stat(self)
    }

    pub fn goal_diff(&self) -> (GoalDiff, GoalDiff) {
        GoalDiff::stat(self)
    }

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

#[derive(
    Default,
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
        let game = PlayedGroupGame::try_new(0, 0, 1, (3, 0), (0, 0), Date::mock()).unwrap();
        let (home, away) = game.points();
        assert_eq!(home, GroupPoint(3));
        assert_eq!(away, GroupPoint(0));
    }

    #[test]
    fn away_win() {
        let game = PlayedGroupGame::try_new(0, 0, 1, (0, 2), (0, 0), Date::mock()).unwrap();
        let (home, away) = game.points();
        assert_eq!(home, GroupPoint(0));
        assert_eq!(away, GroupPoint(3));
    }

    #[test]
    fn draw() {
        let game = PlayedGroupGame::try_new(0, 0, 1, (0, 0), (0, 0), Date::mock()).unwrap();
        let (home, away) = game.points();
        assert_eq!(home, GroupPoint(1));
        assert_eq!(away, GroupPoint(1));
    }
}
