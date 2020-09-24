//! Tournament group play
pub mod game;
pub mod order;
pub mod stats;
use crate::fair_play::FairPlayScore;
use crate::game::{Game, GoalCount, GoalDiff};
use crate::team::{Rank, Team, TeamId};
use crate::Date;
use derive_more::{Add, AddAssign, Display, From};
use game::{PlayedGroupGame, Score, UnplayedGroupGame};
use itertools::Itertools;
pub use order::{order_group, GroupOrder, Rules, Tiebreaker};
use serde::{Deserialize, Serialize};
use stats::UnaryStat;
use std::collections::{BTreeMap, HashMap};
use std::iter;
use thiserror::Error;

/// Type alias for a mapping of `GroupId` to `Group`
pub type Groups = BTreeMap<GroupId, Group>;

/// Group Id
///
/// Uses a `char` as an identifier.
/// At least in football, groups are often labelled with an upper case character.
/// The char is currently limited to ascii alphabetic char's, i.e. A-Z, a-z.
/// This restriction is totally arbitrary and could be lifted, but for now I think it's nice to
/// have it.
#[derive(
    Deserialize,
    Serialize,
    Debug,
    Display,
    Clone,
    Copy,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::cmp::Ord,
    std::cmp::PartialOrd,
    std::hash::Hash,
    From,
)]
pub struct GroupId(char);

impl GroupId {
    pub fn try_new(id: char) -> Result<Self, GroupError> {
        if id.is_ascii_alphabetic() {
            Ok(GroupId(id))
        } else {
            Err(GroupError::InvalidGroupId(id))
        }
    }
}

/// Single group data structure
///
/// The only data that group holds are the games, played and unplayed.
/// Intuitively, one might expect it to hold group stats, whether it is finished, a ranked list of the
/// teams et c.
/// Fundamentally though, the only data are the games. Everything else can be derived from them.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Group {
    played_games: Vec<PlayedGroupGame>,
    upcoming_games: Vec<UnplayedGroupGame>,
}

impl Group {
    /// Fallible `Group` constructor
    ///
    /// Creates a new group from a vector of played and upcoming games.
    /// Imposes the following restrictions on the group type (more might come):
    ///
    /// - Every game (played and upcoming) must have a unique game id.
    pub fn try_new(
        upcoming_games: Vec<UnplayedGroupGame>,
        played_games: Vec<PlayedGroupGame>,
    ) -> Result<Self, GroupError> {
        if Self::game_ids_unique(&played_games, &upcoming_games) {
            Ok(Self {
                upcoming_games,
                played_games,
            })
        } else {
            Err(GroupError::GameIdsNotUnique)
        }
    }

    /// Get teams in group
    ///
    /// Finds all team id's in the group games
    /// (played and upcoming).
    /// Returns an iterator over unique team id's
    pub fn teams(&self) -> impl Iterator<Item = TeamId> + '_ {
        Group::unique_teams_in_games(&self.played_games)
            .chain(Group::unique_teams_in_games(&self.upcoming_games))
            .unique()
    }

    /// Games accessor
    pub fn upcoming_games(&self) -> impl Iterator<Item = &UnplayedGroupGame> {
        self.upcoming_games.iter()
    }

    /// Games accessor
    pub fn played_games(&self) -> impl Iterator<Item = &PlayedGroupGame> {
        self.played_games.iter()
    }

    /// Group size by teams
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

    /// Check games for unique id's
    ///
    /// Extract and combine Game Id's from played and upcoming games.
    /// Compare the number of unique id's with the total number of games
    fn game_ids_unique(
        played_games: &[PlayedGroupGame],
        unplayed_games: &[UnplayedGroupGame],
    ) -> bool {
        let played_ids = played_games.iter().map(|x| x.id);
        let unplayed_ids = unplayed_games.iter().map(|x| x.id);
        let unique_game_ids = played_ids.chain(unplayed_ids).unique();
        unique_game_ids.count() == played_games.len() + unplayed_games.len()
    }

    /// Iterator over unique group teams from list of games
    fn unique_teams_in_games<T: Game>(games: &[T]) -> impl Iterator<Item = TeamId> + '_ {
        games
            .iter()
            .flat_map(|game| iter::once(game.home_team()).chain(iter::once(game.away_team())))
            .unique()
    }
}

/// Group point
///
/// Represents the primary score of a team in a group, either accumulated over multiple games or
/// the outcome of a single game.
/// Commonly, but not necessarily, defined like:
/// - Win: 3 group points
/// - Draw: 1 group point
/// - Loss: 0 group points
#[derive(
    Default, Debug, Display, Clone, Copy, From, Eq, PartialEq, Ord, PartialOrd, Add, AddAssign,
)]
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
    #[error("Group Id '{0}' not an ascii letter (A-Z, a-z)")]
    InvalidGroupId(char),
    #[error("Generic")]
    GenericError,
}

pub fn mock_data() -> (Groups, HashMap<TeamId, Team>) {
    let game_1 = UnplayedGroupGame::try_new(2, 2, 3, Date::mock()).unwrap();
    let game_2 = UnplayedGroupGame::try_new(1, 0, 1, Date::mock())
        .unwrap()
        .play(Score::from((2, 1)), FairPlayScore::from((0, 1)));
    let group_a = Group::try_new(vec![game_1], vec![game_2]).unwrap();
    let game_1 = UnplayedGroupGame::try_new(3, 4, 5, Date::mock()).unwrap();
    let game_2 = UnplayedGroupGame::try_new(4, 6, 7, Date::mock()).unwrap();
    let group_b = Group::try_new(vec![game_1, game_2], vec![]).unwrap();
    let mut groups = BTreeMap::new();
    groups.insert(GroupId('A'), group_a);
    groups.insert(GroupId('B'), group_b);
    let teams = vec![
        Team::new(TeamId(0), "Sweden", "SWE", "se", Rank(0)),
        Team::new(TeamId(1), "England", "ENG", "en", Rank(1)),
        Team::new(TeamId(2), "France", "FRA", "fr", Rank(2)),
        Team::new(TeamId(3), "Brazil", "BRA", "br", Rank(3)),
        Team::new(TeamId(4), "Canada", "CAN", "ca", Rank(4)),
        Team::new(TeamId(5), "Spain", "ESP", "sp", Rank(5)),
        Team::new(TeamId(6), "Japan", "JAP", "ja", Rank(6)),
        Team::new(TeamId(7), "Norway", "NOR", "no", Rank(6)),
    ];
    let teams: HashMap<TeamId, Team> = teams.into_iter().map(|team| (team.id, team)).collect();
    (groups, teams)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fair_play::FairPlayScore;
    use crate::group::game::{Score, UnplayedGroupGame};
    use crate::team::{TeamId, TeamName};
    use crate::Date;
    use std::collections::HashSet;
    #[test]
    fn mock_data_access() {
        let (_, mock_teams) = mock_data();
        assert_eq!(
            mock_teams.get(&TeamId(0)).unwrap().name,
            TeamName(String::from("Sweden"))
        );
    }
    #[test]
    fn group_unique_game_ids_fail() {
        let game_1 = UnplayedGroupGame::try_new(1, 0, 1, Date::mock()).unwrap();
        let game_2 = UnplayedGroupGame::try_new(2, 0, 3, Date::mock()).unwrap();
        let upcoming = vec![game_1, game_2];
        let game_3 = PlayedGroupGame::try_new(2, 2, 1, (1, 2), (0, 1), Date::mock()).unwrap();
        let played = vec![game_3];
        assert_eq!(Group::game_ids_unique(&played, &upcoming), false);
    }
    #[test]
    fn group_unique_game_ids_ok() {
        let game_1 = UnplayedGroupGame::try_new(1, 0, 1, Date::mock()).unwrap();
        let game_2 = UnplayedGroupGame::try_new(2, 0, 3, Date::mock()).unwrap();
        let upcoming = vec![game_1, game_2];
        let game_3 = PlayedGroupGame::try_new(3, 2, 1, (1, 2), (0, 1), Date::mock()).unwrap();
        let played = vec![game_3];
        assert_eq!(Group::game_ids_unique(&played, &upcoming), true);
    }
    #[test]
    fn test_team_from_game_vec() {
        let game_1 = UnplayedGroupGame::try_new(1, 0, 1, Date::mock()).unwrap();
        let game_2 = UnplayedGroupGame::try_new(2, 0, 3, Date::mock()).unwrap();
        let parsed_teams: HashSet<TeamId> =
            Group::unique_teams_in_games(&vec![game_1, game_2]).collect();
        let mut true_teams = HashSet::new();
        true_teams.insert(TeamId(0));
        true_teams.insert(TeamId(1));
        true_teams.insert(TeamId(3));
        assert_eq!(true_teams, parsed_teams)
    }
    #[test]
    fn test_group_teams() {
        let game_1 = UnplayedGroupGame::try_new(1, 0, 1, Date::mock()).unwrap();
        let game_2 = UnplayedGroupGame::try_new(3, 1, 2, Date::mock())
            .unwrap()
            .play(Score::from((2, 0)), FairPlayScore::default());
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
