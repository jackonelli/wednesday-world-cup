use crate::game::group::{PlayedGroupGame, PreGroupGame};
use crate::game::{GoalCount, NumGames};
use crate::team::TeamId;
use derive_more::{Add, AddAssign};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Add, AddAssign)]
pub struct GroupPoint(pub u8);

pub struct Group {
    teams: HashSet<TeamId>,
    upcoming_games: Vec<PreGroupGame>,
    played_games: Vec<PlayedGroupGame>,
}

impl Group {
    pub fn try_new(
        teams: HashSet<TeamId>,
        upcoming_games: Vec<PreGroupGame>,
        played_games: Vec<PlayedGroupGame>,
    ) -> Result<Self, GroupError> {
        if upcoming_games
            .iter()
            .all(|game| teams.contains(&game.home) && teams.contains(&game.away))
            && played_games
                .iter()
                .all(|game| teams.contains(&game.home) && teams.contains(&game.away))
        {
            Ok(Self {
                teams,
                upcoming_games,
                played_games,
            })
        } else {
            Err(GroupError::TeamsNotUnique)
        }
    }

    fn init_group_stats(&self) -> GroupStats {
        self.teams
            .iter()
            .fold(GroupStats::default(), |mut acc, id| {
                acc.0.insert(*id, GroupTeamStats::default());
                acc
            })
    }

    fn rank_teams(&self) -> Vec<TeamId> {
        let stats = self.stats();
        todo!();
    }

    pub fn stats(&self) -> GroupStats {
        self.played_games
            .iter()
            .fold(self.init_group_stats(), |mut acc, game| {
                let (delta_home_stats, delta_away_stats) = game.stats();

                let stats = acc
                    .0
                    .get_mut(&game.home)
                    .expect("TeamId will always be present");
                *stats += delta_home_stats;

                let stats = acc
                    .0
                    .get_mut(&game.away)
                    .expect("TeamId will always be present");
                *stats += delta_away_stats;
                acc
            })
    }
}

#[derive(Default, Debug)]
pub struct GroupStats(HashMap<TeamId, GroupTeamStats>);

impl GroupStats {
    pub fn get(&self, id: &TeamId) -> Option<&GroupTeamStats> {
        self.0.get(id)
    }
    pub fn rank_teams() -> Vec<TeamId> {
        todo!()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Add, AddAssign)]
pub struct GroupTeamStats {
    points: GroupPoint,
    games_played: NumGames,
    goals_scored: GoalCount,
    goals_conceded: GoalCount,
}

impl GroupTeamStats {
    pub fn new(
        points: GroupPoint,
        games_played: NumGames,
        goals_scored: GoalCount,
        goals_conceded: GoalCount,
    ) -> Self {
        GroupTeamStats {
            points,
            games_played,
            goals_scored,
            goals_conceded,
        }
    }
}

#[derive(Error, Debug)]
pub enum GroupError {
    #[error("Teams in group not unique")]
    TeamsNotUnique,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::group::{GroupGameId, Score};
    use crate::game::GoalCount;
    use crate::Date;
    #[test]
    fn test_stats_from_played_games() {
        let teams = [TeamId(0), TeamId(1), TeamId(2), TeamId(3)].iter().fold(
            HashSet::new(),
            |mut teams, team| {
                teams.insert(*team);
                teams
            },
        );
        let score = Score {
            home: GoalCount(2),
            away: GoalCount(1),
        };
        let game_1 = PreGroupGame::new(GroupGameId(1), TeamId(0), TeamId(1), Date {});
        let game_1 = game_1.play(score);

        let score = Score {
            home: GoalCount(0),
            away: GoalCount(0),
        };
        let game_2 = PreGroupGame::new(GroupGameId(2), TeamId(2), TeamId(3), Date {});
        let game_2 = game_2.play(score);

        let score = Score {
            home: GoalCount(2),
            away: GoalCount(1),
        };
        let game_3 = PreGroupGame::new(GroupGameId(3), TeamId(3), TeamId(1), Date {});
        let game_3 = game_3.play(score);

        let games = vec![game_1, game_2, game_3];
        let group = Group::try_new(teams, Vec::new(), games).unwrap();
        let stats = group.stats();
        let is_1 = *stats.get(&TeamId(1)).unwrap();
        let is_2 = *stats.get(&TeamId(2)).unwrap();
        let is_3 = *stats.get(&TeamId(3)).unwrap();

        let true_1 = GroupTeamStats {
            points: GroupPoint(0),
            goals_scored: GoalCount(2),
            goals_conceded: GoalCount(4),
            games_played: NumGames(2),
        };
        let true_2 = GroupTeamStats {
            points: GroupPoint(1),
            goals_scored: GoalCount(0),
            goals_conceded: GoalCount(0),
            games_played: NumGames(1),
        };
        let true_3 = GroupTeamStats {
            points: GroupPoint(4),
            goals_scored: GoalCount(2),
            goals_conceded: GoalCount(1),
            games_played: NumGames(2),
        };
        assert_eq!(is_1, true_1);
        assert_eq!(is_2, true_2);
        assert_eq!(is_3, true_3);
    }
}
