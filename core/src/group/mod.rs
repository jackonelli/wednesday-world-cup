//! # Tournament group stage
pub mod game;
pub mod order;
pub mod stats;
use crate::fair_play::FairPlayScore;
use crate::game::{Game, GameId, GoalCount, GoalDiff, NumGames};
use crate::group::game::GroupGameScore;
use crate::team::TeamId;
use crate::Date;
use derive_more::{Add, AddAssign, Display, From, Into};
use game::{PlayedGroupGame, UnplayedGroupGame};
use itertools::Itertools;
pub use order::{order_group, GroupOrder, Rules, Tiebreaker};
use rand::{
    distributions::Distribution, distributions::Uniform, rngs::StdRng, seq::IteratorRandom,
    thread_rng, SeedableRng,
};
use serde::{de, Deserialize, Deserializer, Serialize};
use stats::GameStat;
use std::collections::HashSet;
use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::iter;
use thiserror::Error;

/// Single group data structure
///
/// The only data that `Group` holds are the games, played and unplayed.
/// Intuitively, one might expect it to hold group stats, whether it is finished, a ranked list of the
/// teams et c.
/// Fundamentally though, the only data are the games. Everything else can be derived from them.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Group {
    played_games: Vec<PlayedGroupGame>,
    unplayed_games: Vec<UnplayedGroupGame>,
}

impl Group {
    /// Fallible `Group` constructor
    ///
    /// Creates a new group from a vector of played and upcoming games.
    ///
    /// # Errors
    ///
    /// The following restrictions on the group type (more might come) are imposed:
    ///
    /// - Every game (played and upcoming) must have a unique game id.
    ///
    /// Returns error variant if any restriction is violated.
    pub fn try_new(
        unplayed_games: Vec<UnplayedGroupGame>,
        played_games: Vec<PlayedGroupGame>,
    ) -> Result<Self, GroupError> {
        if Self::game_ids_unique(&played_games, &unplayed_games) {
            Ok(Self {
                played_games,
                unplayed_games,
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
    pub fn team_ids(&self) -> impl Iterator<Item = TeamId> + '_ {
        Group::unique_teams_in_games(&self.played_games)
            .chain(Group::unique_teams_in_games(&self.unplayed_games))
            .unique()
    }

    /// Games accessor
    pub fn unplayed_games(&self) -> impl Iterator<Item = &UnplayedGroupGame> {
        self.unplayed_games.iter()
    }

    /// Games accessor
    pub fn played_games(&self) -> impl Iterator<Item = &PlayedGroupGame> {
        self.played_games.iter()
    }

    pub fn play_game(&mut self, game_id: GameId, score: GroupGameScore) {
        let idx = self
            .unplayed_games()
            .position(|game| game.id == game_id)
            .unwrap_or_else(|| panic!("No game with id: {:?}", game_id));
        let game = self
            .unplayed_games
            .swap_remove(idx)
            .play(score, FairPlayScore::default());
        self.played_games.push(game);
    }

    pub fn unplay_game(&mut self, game_id: GameId) {
        let idx = self
            .played_games
            .iter()
            .position(|game| game.id == game_id)
            .unwrap();
        let game = self.played_games.swap_remove(idx).unplay();
        self.unplayed_games.push(game);
    }

    /// Group size by teams
    pub fn num_teams(&self) -> usize {
        self.team_ids().count()
    }

    /// Calculate group winner
    ///
    /// Order group according to `rules`
    pub fn rank_teams<T: Tiebreaker>(&self, rules: &Rules<T>) -> GroupOrder {
        order_group(self, rules)
    }

    /// Calculate group winner
    ///
    /// Order group according to `rules` and return first in order.
    pub fn winner<T: Tiebreaker>(&self, rules: &Rules<T>) -> TeamId {
        order_group(self, rules).winner()
    }

    /// Calculate group runner up
    ///
    /// Order group according to `rules` and return second in order.
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

    pub fn random<NG>(
        num_games: NG,
        num_teams: u32,
        min_games_played: NG,
        seed: Option<u64>,
    ) -> Self
    where
        NG: Into<NumGames>,
    {
        let num_games = num_games.into();
        let min_games_played = min_games_played.into();
        let mut rng = match seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_rng(thread_rng()).unwrap(),
        };

        let teams = (0..num_teams).map(TeamId::from);
        let games: Vec<UnplayedGroupGame> = (0..u32::from(num_games))
            .map(|id| {
                let teams = teams.clone().choose_multiple(&mut rng, 2);
                UnplayedGroupGame::try_new(id, teams[0], teams[1], Date::mock()).unwrap()
            })
            .collect();
        let (unpl, pl) = if min_games_played < num_games {
            let (unpl, pl) = games.split_at(
                Uniform::new(
                    u32::from(min_games_played) as usize,
                    u32::from(num_games) as usize,
                )
                .sample(&mut rng),
            );
            (unpl.to_vec(), pl.to_vec())
        } else {
            (Vec::new(), games)
        };
        let pl = pl
            .iter()
            .map(|unpl| {
                let goal_count = Uniform::new(0, 5);
                let score =
                    GroupGameScore::new(goal_count.sample(&mut rng), goal_count.sample(&mut rng));
                unpl.play(score, FairPlayScore::default())
            })
            .collect();
        Group::try_new(unpl.to_vec(), pl).unwrap()
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

/// Group Id
///
/// Uses a `char` as an identifier.
/// At least in football, groups are often labelled with an upper case character.
/// The char is currently limited to ascii alphabetic char's, i.e. A-Z, a-z.
/// This restriction is totally arbitrary and could be lifted, but for now I think it's nice to
/// have it.
#[derive(
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
    Into,
)]
pub struct GroupId(char);

// TODO: remove?
impl GroupId {
    pub fn into_uppercase(self) -> Self {
        Self(self.0.to_ascii_uppercase())
    }
}

impl TryFrom<char> for GroupId {
    type Error = GroupError;
    /// Fallible `GroupId` constructor
    ///
    /// # Errors
    ///
    /// Errors if the arbitrary ascii check fails.
    fn try_from(id: char) -> Result<Self, Self::Error> {
        if id.is_ascii_alphabetic() && id.is_ascii_uppercase() {
            Ok(GroupId(id))
        } else {
            Err(GroupError::InvalidGroupId(id))
        }
    }
}

/// Custom deserializer for [`GroupId`].
/// Parses valid (ASCII alphabetic) char into its uppercase version.
impl<'de> Deserialize<'de> for GroupId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = char::deserialize(deserializer)?;
        if s.is_ascii_alphabetic() {
            Ok(GroupId(s.to_ascii_uppercase()))
        } else {
            Err(de::Error::custom("GroupId must be ASCII"))
        }
    }
}
/// Type alias for a mapping of `GroupId` to `Group`
pub type Groups = BTreeMap<GroupId, Group>;

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

#[derive(Error, Debug, Clone)]
pub enum GroupError {
    #[error("Could not parse group game score '{0}'")]
    GameScoreParse(String),
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

#[derive(Debug, Clone)]
pub enum GroupOutcome {
    Winner(GroupId),
    RunnerUp(GroupId),
    ThirdPlace(HashSet<GroupId>),
}

#[cfg(test)]
pub(crate) mod mock_data {
    use super::*;
    use crate::team::{Team, TeamError, TeamRank, Teams};
    use crate::Date;
    pub fn groups_and_teams() -> (Groups, Teams) {
        let game_1 = UnplayedGroupGame::try_new(2, 3, 4, Date::mock()).unwrap();
        let game_2 = UnplayedGroupGame::try_new(1, 1, 2, Date::mock())
            .unwrap()
            .play(GroupGameScore::from((2, 1)), FairPlayScore::default());
        let group_a = Group::try_new(vec![game_1], vec![game_2]).unwrap();
        let game_1 = UnplayedGroupGame::try_new(3, 5, 6, Date::mock()).unwrap();
        let game_2 = UnplayedGroupGame::try_new(4, 7, 8, Date::mock()).unwrap();
        let group_b = Group::try_new(vec![game_1, game_2], vec![]).unwrap();
        let mut groups = BTreeMap::new();
        groups.insert(GroupId('A'), group_a);
        groups.insert(GroupId('B'), group_b);
        let teams = vec![
            Team::try_new(TeamId(1), "Sweden", "SWE", TeamRank(0)),
            Team::try_new(TeamId(2), "England", "ENG", TeamRank(1)),
            Team::try_new(TeamId(3), "France", "FRA", TeamRank(2)),
            Team::try_new(TeamId(4), "Brazil", "BRA", TeamRank(3)),
            Team::try_new(TeamId(5), "Canada", "CAN", TeamRank(4)),
            Team::try_new(TeamId(6), "Spain", "ESP", TeamRank(5)),
            Team::try_new(TeamId(7), "Japan", "JAP", TeamRank(6)),
            Team::try_new(TeamId(8), "Norway", "NOR", TeamRank(6)),
        ];
        let teams = teams
            .into_iter()
            .collect::<Result<Vec<Team>, TeamError>>()
            .expect("Team creation should not fail");
        let teams: HashMap<TeamId, Team> = teams.into_iter().map(|team| (team.id, team)).collect();
        (groups, teams)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fair_play::FairPlayScore;
    use crate::group::game::{GroupGameScore, UnplayedGroupGame};
    use crate::team::{TeamId, TeamName};
    use crate::Date;
    use std::collections::HashSet;
    #[test]
    fn mock_data_access() {
        let (_, mock_teams) = mock_data::groups_and_teams();
        assert_eq!(
            mock_teams.get(&TeamId(1)).unwrap().name,
            TeamName(String::from("Sweden"))
        );
    }
    #[test]
    fn group_unique_game_ids_fail() {
        let game_1 = UnplayedGroupGame::try_new(1, 0, 1, Date::mock()).unwrap();
        let game_2 = UnplayedGroupGame::try_new(2, 0, 3, Date::mock()).unwrap();
        let upcoming = vec![game_1, game_2];
        let game_3 =
            PlayedGroupGame::try_new(2, 2, 1, (1, 2), FairPlayScore::default(), Date::mock())
                .unwrap();
        let played = vec![game_3];
        assert_eq!(Group::game_ids_unique(&played, &upcoming), false);
    }
    #[test]
    fn group_unique_game_ids_ok() {
        let game_1 = UnplayedGroupGame::try_new(1, 0, 1, Date::mock()).unwrap();
        let game_2 = UnplayedGroupGame::try_new(2, 0, 3, Date::mock()).unwrap();
        let upcoming = vec![game_1, game_2];
        let game_3 =
            PlayedGroupGame::try_new(3, 2, 1, (1, 2), FairPlayScore::default(), Date::mock())
                .unwrap();
        let played = vec![game_3];
        assert_eq!(Group::game_ids_unique(&played, &upcoming), true);
    }
    #[test]
    fn test_team_from_game_vec() {
        let game_1 = UnplayedGroupGame::try_new(1, 0, 1, Date::mock()).unwrap();
        let game_2 = UnplayedGroupGame::try_new(2, 0, 3, Date::mock()).unwrap();
        let parsed_teams: HashSet<TeamId> =
            Group::unique_teams_in_games(&[game_1, game_2]).collect();
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
            .play(GroupGameScore::from((2, 0)), FairPlayScore::default());
        let parsed_teams: HashSet<TeamId> = Group::try_new(vec![game_1], vec![game_2])
            .unwrap()
            .team_ids()
            .collect();
        let mut true_teams = HashSet::new();
        true_teams.insert(TeamId(0));
        true_teams.insert(TeamId(1));
        true_teams.insert(TeamId(2));
        assert_eq!(true_teams, parsed_teams)
    }
}
