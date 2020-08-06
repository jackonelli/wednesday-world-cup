use crate::game::{Game, GoalCount, GoalDiff};
use crate::group::game::{PlayedGroupGame, PreGroupGame};
use crate::group::order::{order_group, GroupOrder, Rules, Tiebreaker};
use crate::group::stats::UnaryStat;
use crate::team::TeamId;
use crate::Date;
use derive_more::{Add, AddAssign, From};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use wasm_bindgen::prelude::*;
pub mod game;
pub mod order;
pub mod stats;

pub type Groups = HashMap<GroupId, Group>;

#[wasm_bindgen]
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

#[wasm_bindgen]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Group {
    upcoming_games: Vec<PreGroupGame>,
    played_games: Vec<PlayedGroupGame>,
}

// Wasm API
#[wasm_bindgen]
impl Group {
    pub fn mock() -> Self {
        let game_1 = PlayedGroupGame::try_new(0, 0, 1, (1, 0), (0, 0), Date::mock()).unwrap();
        let game_2 = PlayedGroupGame::try_new(1, 0, 2, (0, 1), (0, 0), Date::mock()).unwrap();
        let game_3 = PlayedGroupGame::try_new(2, 1, 2, (1, 0), (0, 0), Date::mock()).unwrap();
        let group = Group::try_new(vec![game_1, game_2, game_3], vec![])
            .expect("This literal group construction should never fail");
        group
    }
    pub fn repr(&self) -> String {
        format!(
            "Group:\n\tPlayed games: {}\n\tUpcoming games: {}",
            self.played_games.len(),
            self.upcoming_games.len()
        )
    }
}

impl Group {
    /// Fallible `Group` constructor
    ///
    /// Creates a new group from a vector of played and upcoming games.
    /// Imposes the following restrictions on the group type (more might come):
    ///
    /// - Every game (played or upcoming) must have a unique game id.
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

    /// Check games for unique id's.
    ///
    /// Extract and combine Game Id's from played and upcoming games.
    /// Compare the number of unique id's with the total number of games
    fn unique_game_ids(played_games: &[PlayedGroupGame], upcoming_games: &[PreGroupGame]) -> bool {
        let unique_game_ids: Vec<_> = played_games
            .iter()
            .map(|x| x.id.clone())
            .chain(upcoming_games.iter().map(|x| x.id.clone()))
            .unique()
            .collect();
        unique_game_ids.len() == played_games.len() + upcoming_games.len()
    }

    /// Get teams in group
    ///
    /// Finds all team id's in the group games
    /// (played and upcoming).
    /// Returns an iterator over unique team id's
    pub fn teams(&self) -> impl Iterator<Item = TeamId> {
        team_set_from_game_vec(&self.played_games)
            .chain(team_set_from_game_vec(&self.upcoming_games))
            .unique()
    }

    pub fn num_teams(&self) -> usize {
        self.teams().count()
    }

    /// Calculate group winner
    ///
    /// Order group according to `order_fn`
    pub fn rank_teams<T: Tiebreaker>(&self, rules: &Rules<T>) -> GroupOrder {
        order_group(self, rules)
    }

    /// Calculate group winner
    ///
    /// Order group according to `order_fn` and return first in order.
    pub fn winner<T: Tiebreaker>(&self, rules: &Rules<T>) -> TeamId {
        order_group(self, rules).winner()
    }

    /// Calculate group runner up
    ///
    /// Order group according to `order_fn` and return second in order.
    pub fn runner_up<T: Tiebreaker>(&self, rules: &Rules<T>) -> TeamId {
        order_group(self, rules).runner_up()
    }

    /// Calculate points for group teams
    pub fn points(&self) -> HashMap<TeamId, GroupPoint> {
        GroupPoint::team_stats(self)
    }

    /// Calculate goal difference for group teams
    pub fn goal_diff(&self) -> HashMap<TeamId, GoalDiff> {
        GoalDiff::team_stats(self)
    }

    /// Calculate goals scored for group teams
    pub fn goals_scored(&self) -> HashMap<TeamId, GoalCount> {
        GoalCount::team_stats(self)
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

#[derive(Default, Debug, Clone, Copy, From, Eq, PartialEq, Ord, PartialOrd, Add, AddAssign)]
pub struct GroupPoint(pub u8);

impl num::Zero for GroupPoint {
    fn zero() -> GroupPoint {
        GroupPoint(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Error, Debug)]
pub enum GroupError {
    #[error("Teams in game not unique")]
    GameTeamsNotUnique,
    #[error("Game Id's in group not unique")]
    GameIdsNotUnique,
    #[error("Group does not define a strict ordering")]
    NonStrictOrder,
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
        let game_1 = PreGroupGame::try_new(1, 0, 1, Date::mock()).unwrap();
        let game_2 = PreGroupGame::try_new(2, 0, 3, Date::mock()).unwrap();
        let upcoming = vec![game_1, game_2];
        let game_3 = PlayedGroupGame::try_new(2, 2, 1, (1, 2), (0, 1), Date::mock()).unwrap();
        let played = vec![game_3];
        assert_eq!(Group::unique_game_ids(&played, &upcoming), false);
    }
    #[test]
    fn group_unique_game_ids_ok() {
        let game_1 = PreGroupGame::try_new(1, 0, 1, Date::mock()).unwrap();
        let game_2 = PreGroupGame::try_new(2, 0, 3, Date::mock()).unwrap();
        let upcoming = vec![game_1, game_2];
        let game_3 = PlayedGroupGame::try_new(3, 2, 1, (1, 2), (0, 1), Date::mock()).unwrap();
        let played = vec![game_3];
        assert_eq!(Group::unique_game_ids(&played, &upcoming), true);
    }
    #[test]
    fn test_team_from_game_vec() {
        let game_1 = PreGroupGame::try_new(1, 0, 1, Date::mock()).unwrap();
        let game_2 = PreGroupGame::try_new(2, 0, 3, Date::mock()).unwrap();
        let parsed_teams: HashSet<TeamId> = team_set_from_game_vec(&vec![game_1, game_2]).collect();
        let mut true_teams = HashSet::new();
        true_teams.insert(TeamId(0));
        true_teams.insert(TeamId(1));
        true_teams.insert(TeamId(3));
        assert_eq!(true_teams, parsed_teams)
    }
    #[test]
    fn test_group_teams() {
        let game_1 = PreGroupGame::try_new(3, 1, 2, Date::mock())
            .unwrap()
            .play(Score::new(2, 0), FairPlayScore::default());
        let game_2 = PreGroupGame::try_new(1, 0, 1, Date::mock()).unwrap();
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
