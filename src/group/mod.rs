use crate::game::group;
use crate::game::group::{PlayedGroupGame, PreGroupGame};
use crate::game::{Game, GoalCount, GoalDiff};
use crate::group::order::GroupOrder;
use crate::group::stats::{GroupPoint, PrimaryStats, Unary};
use crate::team::TeamId;
use derive_more::From;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
pub mod order;
pub mod stats;

#[derive(
    Deserialize,
    Serialize,
    Debug,
    Clone,
    Copy,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::hash::Hash,
    From,
)]
pub struct GroupId(pub char);

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

    fn rank_teams(&self, order: fn(&Group) -> GroupOrder) -> GroupOrder {
        order(self)
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

    pub fn primary_stats(&self) -> HashMap<TeamId, PrimaryStats> {
        self.unary_stat(group::primary_stats)
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
    use crate::fair_play::FairPlayScore;
    use crate::game::group::PreGroupGame;
    use crate::game::group::Score;
    use crate::team::TeamId;
    use crate::Date;
    #[test]
    fn test_team_from_game_vec() {
        let game_1 = PreGroupGame::new(1, 0, 1, Date::dummy());
        let game_2 = PreGroupGame::new(2, 0, 3, Date::dummy());
        let parsed_teams: HashSet<TeamId> = team_set_from_game_vec(&vec![game_1, game_2]).collect();
        let mut true_teams = HashSet::new();
        true_teams.insert(TeamId(0));
        true_teams.insert(TeamId(1));
        true_teams.insert(TeamId(3));
        assert_eq!(true_teams, parsed_teams)
    }
    #[test]
    fn test_group_teams() {
        let game_1 = PreGroupGame::new(1, 0, 1, Date::dummy());
        let game_2 = PreGroupGame::new(3, 1, 2, Date::dummy())
            .play(Score::new(2, 0), FairPlayScore::default());
        let parsed_teams: HashSet<TeamId> = Group::try_new(vec![game_1], vec![game_2])
            .unwrap()
            .teams()
            .collect();
        let mut true_teams = HashSet::new();
        true_teams.insert(TeamId(0));
        true_teams.insert(TeamId(1));
        true_teams.insert(TeamId(2));
        assert_eq!(true_teams, parsed_teams)
    }
}
