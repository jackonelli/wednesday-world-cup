//! Tournament playoff
//!
//! This module models the knockout stage of a tournament using a functional,
//! graph-based approach.
//!
//! # Architecture
//!
//! The playoff system separates concerns into:
//!
//! - **Structure** ([`bracket::BracketStructure`]): The static graph topology
//!   - Defined once per tournament format (e.g., "Euro 2020", "World Cup 2018")
//!   - Represents which games exist and how teams flow between them
//!   - Immutable after construction
//!
//! - **State** ([`bracket::BracketState`]): Immutable snapshots of predictions
//!   - Contains only the played games and their scores
//!   - Purely functional - all operations return new states
//!   - Easy to serialize, undo/redo, compare
//!
//! - **View** ([`bracket::PlayoffGameState`]): Computed views
//!   - Derived by combining structure + state
//!   - Shows current status of each game (Pending, Ready, Played, etc.)
//!   - Never stored, always computed on demand
//!
//! # Graph Model
//!
//! The bracket is a directed acyclic graph (DAG) where:
//! - **Nodes** = Playoff games
//! - **Edges** = Team progression (winner/loser flows to next game)
//!
//! This allows modeling complex brackets like:
//! - Standard single elimination
//! - Third-place playoffs
//! - Winners vs. losers progression
//!
//! # Example
//!
//! ```rust,ignore
//! use wwc_core::playoff::bracket::{BracketStructure, BracketState, BracketTemplate};
//!
//! // Define bracket structure (once per tournament)
//! let template = BracketTemplate::euro_2020();
//! let bracket = BracketStructure::from_template(template)?;
//!
//! // Start with empty state
//! let mut state = BracketState::new();
//!
//! // View what's ready to play
//! let ready_games = bracket.all_game_states(&state, &groups, &rules)
//!     .into_iter()
//!     .filter(|g| g.is_ready())
//!     .collect::<Vec<_>>();
//!
//! // Play a game (functional - returns new state)
//! state = state.play_game(game_id, home_team, away_team, score);
//!
//! // Check who won the tournament
//! if let Some(champion) = bracket.champion(&state) {
//!     println!("Champion: {}", champion);
//! }
//! ```

pub mod bracket;
pub mod game;
pub mod template;
pub mod transition;

// Re-exports for convenience
pub use bracket::{
    BracketError, BracketState, BracketStructure, PlayoffGameState, PlayoffResult, TeamSource,
};
pub use game::{PlayoffError, PlayoffScore};
use template::BracketTemplate;

use crate::game::GameId;
use crate::group::Groups;
use crate::group::order::{Rules, Tiebreaker};
use crate::team::TeamId;

/// High-level playoff predictions
///
/// Combines a bracket structure with user predictions.
/// This is the main entry point for working with playoffs.
pub struct PlayoffPredictions {
    structure: BracketStructure,
    state: BracketState,
}

impl PlayoffPredictions {
    /// Create new playoff predictions from a template
    pub fn new(template: BracketTemplate) -> Result<Self, BracketError> {
        let structure = BracketStructure::from_template(template)?;
        let state = BracketState::new();
        Ok(Self { structure, state })
    }

    /// Create from existing structure and state
    pub fn from_parts(structure: BracketStructure, state: BracketState) -> Self {
        Self { structure, state }
    }

    /// Get all game states
    pub fn game_states<T: Tiebreaker>(
        &self,
        groups: &Groups,
        rules: &Rules<T>,
    ) -> Vec<PlayoffGameState> {
        self.structure.all_game_states(&self.state, groups, rules)
    }

    /// Get games at a specific round (depth from final)
    pub fn games_at_depth<T: Tiebreaker>(
        &self,
        depth: usize,
        groups: &Groups,
        rules: &Rules<T>,
    ) -> Vec<PlayoffGameState> {
        self.structure
            .games_at_depth(depth, &self.state, groups, rules)
    }

    /// Play a game
    pub fn play_game(&mut self, game_id: GameId, home: TeamId, away: TeamId, score: PlayoffScore) {
        self.state = self.state.play_game(game_id, home, away, score);
    }

    /// Unplay a game
    pub fn unplay_game(&mut self, game_id: GameId) {
        self.state = self.state.unplay_game(game_id);
    }

    /// Get the champion (if final is played)
    pub fn champion(&self) -> Option<TeamId> {
        self.structure.champion(&self.state)
    }

    /// Get the runner-up (if final is played)
    pub fn runner_up(&self) -> Option<TeamId> {
        self.structure.runner_up(&self.state)
    }

    /// Get the current state (for serialization)
    pub fn state(&self) -> &BracketState {
        &self.state
    }

    /// Get the structure (for querying topology)
    pub fn structure(&self) -> &BracketStructure {
        &self.structure
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group::mock_data::groups_and_teams;
    use crate::group::order::fifa_2018_rules;
    use crate::group::{GroupId, GroupOutcome};

    #[test]
    fn test_playoff_predictions() {
        let (groups, _teams) = groups_and_teams();
        let rules = fifa_2018_rules();

        // Create a simple bracket
        let template = BracketTemplate {
            games: vec![(
                GameId::from(1),
                (
                    TeamSource::GroupOutcome(GroupOutcome::Winner(GroupId::try_from('A').unwrap())),
                    TeamSource::GroupOutcome(GroupOutcome::RunnerUp(
                        GroupId::try_from('B').unwrap(),
                    )),
                ),
            )],
            final_game_id: GameId::from(1),
        };

        let predictions = PlayoffPredictions::new(template).unwrap();
        let games = predictions.game_states(&groups, &rules);

        assert_eq!(games.len(), 1);
        assert!(games[0].is_ready());
    }
}
