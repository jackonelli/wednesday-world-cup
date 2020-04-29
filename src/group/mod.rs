use crate::game::group;
use crate::game::group::{PlayedGroupGame, PreGroupGame};
use crate::game::{Game, GoalCount, GoalDiff};
use crate::group::order::GroupOrder;
use crate::group::stats::{GroupPoint, GroupStats, GroupTeamStats, Unary};
use crate::team::TeamId;
use std::collections::{HashMap, HashSet};
use thiserror::Error;
pub mod order;
pub mod stats;

pub struct Group {
    upcoming_games: Vec<PreGroupGame>,
    played_games: Vec<PlayedGroupGame>,
}

impl Group {
    pub fn try_new(
        upcoming_games: Vec<PreGroupGame>,
        played_games: Vec<PlayedGroupGame>,
    ) -> Result<Self, GroupError> {
        // TODO: Check for completeness
        Ok(Self {
            upcoming_games,
            played_games,
        })
    }

    pub fn teams(&self) -> impl Iterator<Item = TeamId> {
        let mut unique_set: HashSet<TeamId> = team_set_from_game_vec(&self.played_games).collect();
        let upcoming_teams = team_set_from_game_vec(&self.upcoming_games);
        unique_set.extend(upcoming_teams);
        unique_set.into_iter()
    }

    fn init_group_stats(&self) -> GroupStats {
        self.teams().fold(GroupStats::default(), |mut acc, id| {
            acc.0.insert(id, GroupTeamStats::default());
            acc
        })
    }

    fn rank_teams(&self, order: fn(&Group) -> GroupOrder) -> Vec<TeamId> {
        let group_ranking = order(self);
        todo!();
    }

    pub fn points(&self) -> HashMap<TeamId, GroupPoint> {
        self.unary_stat(group::points)
    }

    pub fn goal_diff(&self) -> HashMap<TeamId, GoalDiff> {
        self.unary_stat(group::goal_diff)
    }

    pub fn goals_scored(&self) -> HashMap<TeamId, GoalCount> {
        self.unary_stat(group::goals_scored)
    }

    fn unary_stat<T>(&self, stat: fn(&PlayedGroupGame) -> (T, T)) -> HashMap<TeamId, T>
    where
        T: Unary + num::Zero + std::ops::AddAssign,
    {
        let team_map = self.teams().fold(HashMap::new(), |mut acc, team| {
            acc.insert(team, T::zero());
            acc
        });

        self.team_stat_from_played_games(team_map, stat)
    }

    fn team_stat_from_played_games<T>(
        &self,
        default_map: HashMap<TeamId, T>,
        stat: fn(&PlayedGroupGame) -> (T, T),
    ) -> HashMap<TeamId, T>
    where
        T: std::ops::AddAssign,
    {
        self.played_games.iter().fold(default_map, |mut acc, game| {
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

    ///Remove?
    pub fn stats(&self) -> GroupStats {
        let team_map = self.init_group_stats().0;
        GroupStats(self.team_stat_from_played_games(team_map, group::stats))
    }
}

fn team_set_from_game_vec<T: Game>(games: &[T]) -> impl Iterator<Item = TeamId> {
    let teams: HashSet<TeamId> = games.iter().fold(HashSet::default(), |mut acc, game| {
        acc.insert(game.home_team());
        acc.insert(game.away_team());
        acc
    });
    teams.into_iter()
}

#[derive(Error, Debug)]
pub enum GroupError {
    #[error("Teams in group not unique")]
    GameTeamsNotInTeams,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fair_play::FairPlay;
    use crate::game::group::{GroupGameId, PreGroupGame, Score};
    use crate::group::stats::GamesDiff;
    use crate::team::TeamId;
    use crate::Date;
    #[test]
    fn test_team_from_game_vec() {
        let game_1 = PreGroupGame::new(GroupGameId(1), TeamId(0), TeamId(1), Date {});
        let game_2 = PreGroupGame::new(GroupGameId(2), TeamId(0), TeamId(3), Date {});
        let parsed_teams: HashSet<TeamId> = team_set_from_game_vec(&vec![game_1, game_2]).collect();
        let mut true_teams = HashSet::new();
        true_teams.insert(TeamId(0));
        true_teams.insert(TeamId(1));
        true_teams.insert(TeamId(3));
        assert_eq!(true_teams, parsed_teams)
    }
    #[test]
    fn test_group_teams() {
        let game_1 = PreGroupGame::new(GroupGameId(1), TeamId(0), TeamId(1), Date {});
        let game_2 = PreGroupGame::new(GroupGameId(2), TeamId(0), TeamId(3), Date {});
        let parsed_teams: HashSet<TeamId> = Group::try_new(vec![game_1, game_2], vec![])
            .unwrap()
            .teams()
            .collect();
        let mut true_teams = HashSet::new();
        true_teams.insert(TeamId(0));
        true_teams.insert(TeamId(1));
        true_teams.insert(TeamId(3));
        assert_eq!(true_teams, parsed_teams)
    }
    #[test]
    fn stats_from_played_games() {
        let dummy_fair_play = FairPlay::default();
        let score = Score::new(2, 1);
        let game_1 = PreGroupGame::new(GroupGameId(1), TeamId(0), TeamId(1), Date {});
        let game_1 = game_1.play(score, dummy_fair_play.clone());

        let score = Score::new(0, 0);
        let game_2 = PreGroupGame::new(GroupGameId(2), TeamId(2), TeamId(3), Date {});
        let game_2 = game_2.play(score, dummy_fair_play.clone());

        let score = Score::new(2, 1);
        let game_3 = PreGroupGame::new(GroupGameId(3), TeamId(3), TeamId(1), Date {});
        let game_3 = game_3.play(score, dummy_fair_play.clone());

        let games = vec![game_1, game_2, game_3];
        let group = Group::try_new(Vec::new(), games).unwrap();
        let stats = group.stats();
        let is_1 = stats.get(&TeamId(1)).unwrap();
        let is_2 = stats.get(&TeamId(2)).unwrap();
        let is_3 = stats.get(&TeamId(3)).unwrap();

        let true_1 = &GroupTeamStats::new(0, 2, 2, 4, 0, GamesDiff::default());
        let true_2 = &GroupTeamStats::new(1, 1, 0, 0, 0, GamesDiff::default());
        let true_3 = &GroupTeamStats::new(4, 2, 2, 1, 0, GamesDiff::default());
        assert_eq!(is_1, true_1);
        assert_eq!(is_2, true_2);
        assert_eq!(is_3, true_3);
    }
}
