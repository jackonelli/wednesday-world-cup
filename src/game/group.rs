use crate::game::GoalCount;
use crate::game::NumGames;
use crate::group::GroupPoint;
use crate::group::GroupTeamStats;
use crate::team::TeamId;
use crate::Date;
use derive_more::{Add, AddAssign};

#[derive(Clone)]
pub struct Score {
    pub home: GoalCount,
    pub away: GoalCount,
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
    pub fn play(self, score: Score) -> PlayedGroupGame {
        PlayedGroupGame {
            id: self.id,
            home: self.home,
            away: self.away,
            date: self.date,
            score,
        }
    }
}

#[derive(Clone)]
pub struct PlayedGroupGame {
    id: GroupGameId,
    pub home: TeamId,
    pub away: TeamId,
    pub score: Score,
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

    pub(crate) fn stats(&self) -> (GroupTeamStats, GroupTeamStats) {
        let (home_points, away_points) = self.points_rewarded();
        let home_stats =
            GroupTeamStats::new(home_points, NumGames(1), self.score.home, self.score.away);
        let away_stats =
            GroupTeamStats::new(away_points, NumGames(1), self.score.away, self.score.home);
        (home_stats, away_stats)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Add, AddAssign)]
pub struct GroupGameId(pub u8);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::GoalCount;
    #[test]
    fn test_home_win() {
        let game = PlayedGroupGame {
            id: GroupGameId(0),
            home: TeamId(0),
            away: TeamId(1),
            score: Score {
                home: GoalCount(3),
                away: GoalCount(0),
            },
            date: Date {},
        };
        let (home, away) = game.points_rewarded();
        assert_eq!(home, GroupPoint(3));
        assert_eq!(away, GroupPoint(0));
    }

    #[test]
    fn test_away_win() {
        let game = PlayedGroupGame {
            id: GroupGameId(0),
            home: TeamId(0),
            away: TeamId(1),
            score: Score {
                home: GoalCount(0),
                away: GoalCount(2),
            },
            date: Date {},
        };
        let (home, away) = game.points_rewarded();
        assert_eq!(home, GroupPoint(0));
        assert_eq!(away, GroupPoint(3));
    }

    #[test]
    fn test_draw() {
        let game = PlayedGroupGame {
            id: GroupGameId(0),
            home: TeamId(0),
            away: TeamId(1),
            score: Score {
                home: GoalCount(0),
                away: GoalCount(0),
            },
            date: Date {},
        };
        let (home, away) = game.points_rewarded();
        assert_eq!(home, GroupPoint(1));
        assert_eq!(away, GroupPoint(1));
    }
}
