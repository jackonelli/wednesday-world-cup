use crate::fair_play::FairPlayScore;
use crate::game::GoalDiff;
use crate::game::{Game, GoalCount};
use crate::group::stats::{GroupPoint, PrimaryStats};
use crate::group::GroupError;
use crate::team::TeamId;
use crate::Date;
use derive_more::{Add, AddAssign, From};
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
    id: GroupGameId,
    pub home: TeamId,
    pub away: TeamId,
    date: Date,
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
    id: GroupGameId,
    pub home: TeamId,
    pub away: TeamId,
    pub score: Score,
    fair_play: FairPlayScore,
    date: Date,
}

impl PlayedGroupGame {
    pub fn try_new<
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
    pub(crate) fn points(&self) -> (GroupPoint, GroupPoint) {
        let score = &self.score;
        if score.home > score.away {
            (GroupPoint(3), GroupPoint(0))
        } else if score.home < score.away {
            (GroupPoint(0), GroupPoint(3))
        } else {
            (GroupPoint(1), GroupPoint(1))
        }
    }

    fn goal_diff(&self) -> (GoalDiff, GoalDiff) {
        goal_diff(self)
    }

    fn goals_scored(&self) -> (GoalCount, GoalCount) {
        goals_scored(self)
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

pub fn points(game: &PlayedGroupGame) -> (GroupPoint, GroupPoint) {
    let score = &game.score;
    if score.home > score.away {
        (GroupPoint(3), GroupPoint(0))
    } else if score.home < score.away {
        (GroupPoint(0), GroupPoint(3))
    } else {
        (GroupPoint(1), GroupPoint(1))
    }
}

pub fn goal_diff(game: &PlayedGroupGame) -> (GoalDiff, GoalDiff) {
    let goal_diff = game.score.home - game.score.away;
    (goal_diff, -goal_diff)
}

pub fn goals_scored(game: &PlayedGroupGame) -> (GoalCount, GoalCount) {
    (game.score.home, game.score.away)
}

pub fn primary_stats(game: &PlayedGroupGame) -> (PrimaryStats, PrimaryStats) {
    let (home_points, away_points) = game.points();
    let (home_goal_diff, away_goal_diff) = game.goal_diff();
    let (home_goals_scored, away_goals_scored) = game.goals_scored();
    let prim_stats_home = PrimaryStats::new(home_points, home_goal_diff, home_goals_scored);
    let prim_stats_away = PrimaryStats::new(away_points, away_goal_diff, away_goals_scored);
    (prim_stats_home, prim_stats_away)
}

#[derive(
    Default, Debug, Deserialize, Serialize, Clone, Copy, PartialEq, PartialOrd, Add, AddAssign, From,
)]
pub struct GroupGameId(pub u8);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn home_win() {
        let game = PlayedGroupGame::try_new(0, 0, 1, (3, 0), (0, 0), Date::dummy()).unwrap();
        let (home, away) = game.points();
        assert_eq!(home, GroupPoint(3));
        assert_eq!(away, GroupPoint(0));
    }

    #[test]
    fn away_win() {
        let game = PlayedGroupGame::try_new(0, 0, 1, (0, 2), (0, 0), Date::dummy()).unwrap();
        let (home, away) = game.points();
        assert_eq!(home, GroupPoint(0));
        assert_eq!(away, GroupPoint(3));
    }

    #[test]
    fn draw() {
        let game = PlayedGroupGame::try_new(0, 0, 1, (0, 0), (0, 0), Date::dummy()).unwrap();
        let (home, away) = game.points();
        assert_eq!(home, GroupPoint(1));
        assert_eq!(away, GroupPoint(1));
    }
}
