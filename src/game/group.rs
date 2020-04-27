use crate::fair_play::FairPlay;
use crate::game::GoalDiff;
use crate::game::{Game, GoalCount, NumGames};
use crate::group::stats::{GamesDiff, GroupPoint, GroupTeamStats};
use crate::team::TeamId;
use crate::Date;
use derive_more::{Add, AddAssign};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Score {
    pub home: GoalCount,
    pub away: GoalCount,
}

impl Score {
    pub fn new<T: Into<GoalCount>>(home: T, away: T) -> Self {
        Score {
            home: home.into(),
            away: away.into(),
        }
    }
}

pub struct PreGroupGame {
    id: GroupGameId,
    pub home: TeamId,
    pub away: TeamId,
    date: Date,
}

impl PreGroupGame {
    pub fn new(id: GroupGameId, home: TeamId, away: TeamId, date: Date) -> Self {
        Self {
            id,
            home,
            away,
            date,
        }
    }
    pub fn play(self, score: Score, fair_play: FairPlay) -> PlayedGroupGame {
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

impl Game for PlayedGroupGame {
    fn home_team(&self) -> TeamId {
        self.home
    }
    fn away_team(&self) -> TeamId {
        self.away
    }
}

#[derive(Clone)]
pub struct PlayedGroupGame {
    id: GroupGameId,
    pub home: TeamId,
    pub away: TeamId,
    pub score: Score,
    fair_play: FairPlay,
    date: Date,
}

impl PlayedGroupGame {
    pub(crate) fn points_rewarded(&self) -> (GroupPoint, GroupPoint) {
        let score = &self.score;
        if score.home > score.away {
            (GroupPoint(3), GroupPoint(0))
        } else if score.home < score.away {
            (GroupPoint(0), GroupPoint(3))
        } else {
            (GroupPoint(1), GroupPoint(1))
        }
    }

    fn goal_diff(&self) -> GoalDiff {
        self.score.home - self.score.away
    }

    fn game_diff(&self) -> (GamesDiff, GamesDiff) {
        let goal_diff = self.goal_diff();
        let mut home = HashMap::new();
        home.insert(self.away_team(), goal_diff);
        let mut away = HashMap::new();
        away.insert(self.home_team(), -goal_diff);
        (home.into(), away.into())
    }

    pub(crate) fn stats(&self) -> (GroupTeamStats, GroupTeamStats) {
        let (home_points, away_points) = self.points_rewarded();
        let (home_game_diff, away_game_diff) = self.game_diff();
        let home_stats = GroupTeamStats::new(
            home_points,
            NumGames(1),
            self.score.home,
            self.score.away,
            0,
            home_game_diff,
        );
        let away_stats = GroupTeamStats::new(
            away_points,
            NumGames(1),
            self.score.away,
            self.score.home,
            0,
            away_game_diff,
        );
        (home_stats, away_stats)
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

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Add, AddAssign)]
pub struct GroupGameId(pub u8);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::GoalCount;
    #[test]
    fn home_win() {
        let game = PlayedGroupGame {
            id: GroupGameId(0),
            home: TeamId(0),
            away: TeamId(1),
            score: Score {
                home: GoalCount(3),
                away: GoalCount(0),
            },
            fair_play: FairPlay::default(),
            date: Date {},
        };
        let (home, away) = game.points_rewarded();
        assert_eq!(home, GroupPoint(3));
        assert_eq!(away, GroupPoint(0));
    }

    #[test]
    fn away_win() {
        let game = PlayedGroupGame {
            id: GroupGameId(0),
            home: TeamId(0),
            away: TeamId(1),
            score: Score {
                home: GoalCount(0),
                away: GoalCount(2),
            },
            fair_play: FairPlay::new(0, 0, 0, 0),
            date: Date {},
        };
        let (home, away) = game.points_rewarded();
        assert_eq!(home, GroupPoint(0));
        assert_eq!(away, GroupPoint(3));
    }

    #[test]
    fn draw() {
        let game = PlayedGroupGame {
            id: GroupGameId(0),
            home: TeamId(0),
            away: TeamId(1),
            score: Score {
                home: GoalCount(0),
                away: GoalCount(0),
            },
            fair_play: FairPlay::new(0, 0, 0, 0),
            date: Date {},
        };
        let (home, away) = game.points_rewarded();
        assert_eq!(home, GroupPoint(1));
        assert_eq!(away, GroupPoint(1));
    }
}
