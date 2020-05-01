use crate::game::{Game, GoalCount, GoalDiff};
use crate::group::game::{PlayedGroupGame, PreGroupGame};
use crate::group::order::{GroupOrder, GroupRank};
use crate::group::stats::{GroupPoint, PrimaryStats, Unary};
use crate::team::TeamId;
use derive_more::From;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
pub mod game;
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Group {
    upcoming_games: Vec<PreGroupGame>,
    played_games: Vec<PlayedGroupGame>,
}

impl Group {
    pub fn try_new(
        played_games: Vec<PlayedGroupGame>,
        upcoming_games: Vec<PreGroupGame>,
    ) -> Result<Self, GroupError> {
        if Self::unique_game_ids(&played_games, &upcoming_games) {
            Ok(Self {
                upcoming_games,
                played_games,
            })
        } else {
            Err(GroupError::GameIdsNotUnique)
        }
    }

    fn unique_game_ids(played_games: &[PlayedGroupGame], upcoming_games: &[PreGroupGame]) -> bool {
        let games_id: Vec<_> = played_games
            .iter()
            .map(|x| x.id.clone())
            .chain(upcoming_games.iter().map(|x| x.id.clone()))
            .unique()
            .collect();
        games_id.len() == played_games.len() + upcoming_games.len()
    }

    pub fn teams(&self) -> impl Iterator<Item = TeamId> {
        let mut unique_set: HashSet<TeamId> = team_set_from_game_vec(&self.played_games).collect();
        let upcoming_teams = team_set_from_game_vec(&self.upcoming_games);
        unique_set.extend(upcoming_teams);
        unique_set.into_iter()
    }

    pub fn rank_teams(&self, order_fn: fn(&Group) -> GroupOrder) -> GroupOrder {
        order_fn(self)
    }

    pub fn winner(&self, order_fn: fn(&Group) -> GroupOrder) -> TeamId {
        (order_fn(self))[GroupRank(0)]
    }

    pub fn runner_up(&self, order_fn: fn(&Group) -> GroupOrder) -> TeamId {
        (order_fn(self))[GroupRank(1)]
    }

    pub fn points(&self) -> HashMap<TeamId, GroupPoint> {
        self.unary_stat(game::points)
    }

    pub fn goal_diff(&self) -> HashMap<TeamId, GoalDiff> {
        self.unary_stat(game::goal_diff)
    }

    pub fn goals_scored(&self) -> HashMap<TeamId, GoalCount> {
        self.unary_stat(game::goals_scored)
    }

    pub fn primary_stats(&self) -> HashMap<TeamId, PrimaryStats> {
        self.unary_stat(game::primary_stats)
    }

    fn unary_stat<T>(&self, stat: fn(&PlayedGroupGame) -> (T, T)) -> HashMap<TeamId, T>
    where
        T: Unary + num::Zero + std::ops::AddAssign,
    {
        let team_map = self.teams().map(|team| (team, T::zero())).collect();

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
    #[error("Teams in game not unique")]
    GameTeamsNotUnique,
    #[error("Game Id's in group not unique")]
    GameIdsNotUnique,
    #[error("Generic")]
    GenericError,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fair_play::FairPlayScore;
    use crate::group::game::{PreGroupGame, Score};
    use crate::team::TeamId;
    use crate::Date;
    #[test]
    fn group_unique_game_ids_fail() {
        let game_1 = PreGroupGame::try_new(1, 0, 1, Date::dummy()).unwrap();
        let game_2 = PreGroupGame::try_new(2, 0, 3, Date::dummy()).unwrap();
        let upcoming = vec![game_1, game_2];
        let game_3 = PlayedGroupGame::try_new(2, 2, 1, (1, 2), (0, 1), Date::dummy()).unwrap();
        let played = vec![game_3];
        assert_eq!(Group::unique_game_ids(&played, &upcoming), false);
    }
    #[test]
    fn group_unique_game_ids_ok() {
        let game_1 = PreGroupGame::try_new(1, 0, 1, Date::dummy()).unwrap();
        let game_2 = PreGroupGame::try_new(2, 0, 3, Date::dummy()).unwrap();
        let upcoming = vec![game_1, game_2];
        let game_3 = PlayedGroupGame::try_new(3, 2, 1, (1, 2), (0, 1), Date::dummy()).unwrap();
        let played = vec![game_3];
        assert_eq!(Group::unique_game_ids(&played, &upcoming), true);
    }
    #[test]
    fn test_team_from_game_vec() {
        let game_1 = PreGroupGame::try_new(1, 0, 1, Date::dummy()).unwrap();
        let game_2 = PreGroupGame::try_new(2, 0, 3, Date::dummy()).unwrap();
        let parsed_teams: HashSet<TeamId> = team_set_from_game_vec(&vec![game_1, game_2]).collect();
        let mut true_teams = HashSet::new();
        true_teams.insert(TeamId(0));
        true_teams.insert(TeamId(1));
        true_teams.insert(TeamId(3));
        assert_eq!(true_teams, parsed_teams)
    }
    #[test]
    fn test_group_teams() {
        let game_1 = PreGroupGame::try_new(3, 1, 2, Date::dummy())
            .unwrap()
            .play(Score::new(2, 0), FairPlayScore::default());
        let game_2 = PreGroupGame::try_new(1, 0, 1, Date::dummy()).unwrap();
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
