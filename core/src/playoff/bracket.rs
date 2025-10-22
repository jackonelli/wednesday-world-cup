//! Functional playoff bracket model using a directed graph
//!
//! The playoff bracket is modeled as a directed acyclic graph where:
//! - Nodes represent playoff games
//! - Edges represent team progression (winner/loser flows to next game)
//!
//! This module separates:
//! - **Structure** (`BracketStructure`): The static graph topology, defined once per tournament
//! - **State** (`BracketState`): Immutable snapshots of played games
//! - **View** (`PlayoffGameState`): Computed views derived from structure + state

use crate::game::GameId;
use crate::group::order::{Rules, Tiebreaker};
use crate::group::{GroupOutcome, Groups};
use crate::playoff::game::PlayoffScore;
use crate::playoff::template::BracketTemplate;
use crate::team::TeamId;
use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use thiserror::Error;

/// Where a team comes from in the bracket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TeamSource {
    /// Team from group stage outcome
    GroupOutcome(GroupOutcome),
    /// Winner of a previous playoff game
    WinnerOf(GameId),
    /// Loser of a previous playoff game (for 3rd place playoff)
    LoserOf(GameId),
}

impl Display for TeamSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TeamSource::GroupOutcome(outcome) => write!(f, "Group {}", outcome),
            TeamSource::WinnerOf(game_id) => write!(f, "Winner of {}", game_id),
            TeamSource::LoserOf(game_id) => write!(f, "Loser of {}", game_id),
        }
    }
}

/// Edge type in the bracket graph
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    /// Winner advances
    Winner,
    /// Loser advances (for 3rd place playoff)
    Loser,
}

/// Result of a completed playoff game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayoffResult {
    pub home: TeamId,
    pub away: TeamId,
    pub score: PlayoffScore,
}

impl PlayoffResult {
    pub fn new(home: TeamId, away: TeamId, score: PlayoffScore) -> Self {
        Self { home, away, score }
    }

    /// Get the winner of this game
    pub fn winner(&self) -> TeamId {
        self.score.winner(self.home, self.away)
    }

    /// Get the loser of this game
    pub fn loser(&self) -> TeamId {
        self.score.loser(self.home, self.away)
    }
}

/// Immutable snapshot of bracket state
///
/// Contains only the played games. All other information is derived
/// by combining this with the `BracketStructure`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BracketState {
    /// Results of played games
    results: HashMap<GameId, PlayoffResult>,
}

impl BracketState {
    /// Create an empty bracket state
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }

    /// Play a game (returns new state, purely functional)
    pub fn play_game(
        &self,
        game_id: GameId,
        home: TeamId,
        away: TeamId,
        score: PlayoffScore,
    ) -> Self {
        let mut new_results = self.results.clone();
        new_results.insert(game_id, PlayoffResult::new(home, away, score));
        Self {
            results: new_results,
        }
    }

    /// Unplay a game (returns new state, purely functional)
    pub fn unplay_game(&self, game_id: GameId) -> Self {
        let mut new_results = self.results.clone();
        new_results.remove(&game_id);
        Self {
            results: new_results,
        }
    }

    /// Get result of a game (if played)
    pub fn result(&self, game_id: GameId) -> Option<&PlayoffResult> {
        self.results.get(&game_id)
    }

    /// Check if a game has been played
    pub fn is_played(&self, game_id: GameId) -> bool {
        self.results.contains_key(&game_id)
    }

    /// Get all played games
    pub fn played_games(&self) -> impl Iterator<Item = (&GameId, &PlayoffResult)> {
        self.results.iter()
    }

    /// Count of played games
    pub fn num_played(&self) -> usize {
        self.results.len()
    }
}

/// The static structure of a playoff bracket
///
/// Defines the graph topology and team sources for each game.
/// This is immutable and defined once per tournament format.
#[derive(Debug, Clone)]
pub struct BracketStructure {
    /// The directed graph (nodes are GameIds, edges are team flows)
    graph: DiGraph<GameId, EdgeType>,
    /// Team sources for each game
    sources: HashMap<NodeIndex, (TeamSource, TeamSource)>,
    /// The final game node
    final_node: NodeIndex,
    /// Mapping from GameId to NodeIndex for lookups
    game_to_node: HashMap<GameId, NodeIndex>,
}

impl BracketStructure {
    /// Create bracket structure from a template
    pub fn from_template(template: BracketTemplate) -> Result<Self, BracketError> {
        let mut graph = DiGraph::new();
        let mut sources = HashMap::new();
        let mut game_to_node = HashMap::new();

        // First pass: create all nodes
        for (game_id, (home_src, away_src)) in &template.games {
            let node_idx = graph.add_node(*game_id);
            sources.insert(node_idx, (home_src.clone(), away_src.clone()));
            game_to_node.insert(*game_id, node_idx);
        }

        // Second pass: create edges based on dependencies
        for (node_idx, (home_src, away_src)) in &sources {
            Self::add_edge_for_source(&mut graph, home_src.clone(), *node_idx, &game_to_node)?;
            Self::add_edge_for_source(&mut graph, away_src.clone(), *node_idx, &game_to_node)?;
        }

        let final_node = game_to_node
            .get(&template.final_game_id)
            .copied()
            .ok_or(BracketError::FinalGameNotFound)?;

        // Validate: graph should be acyclic
        if petgraph::algo::is_cyclic_directed(&graph) {
            return Err(BracketError::CyclicBracket);
        }

        Ok(Self {
            graph,
            sources,
            final_node,
            game_to_node,
        })
    }

    /// Create bracket structure from team sources
    pub fn from_team_sources(
        team_sources: &[(GameId, (TeamSource, TeamSource))],
    ) -> Result<Self, BracketError> {
        let templ = BracketTemplate {
            games: team_sources.to_vec(),
            final_game_id: team_sources
                .last()
                .ok_or(BracketError::FinalGameNotFound)?
                .0,
        };
        Self::from_template(templ)
    }

    fn add_edge_for_source(
        graph: &mut DiGraph<GameId, EdgeType>,
        source: TeamSource,
        target: NodeIndex,
        game_to_node: &HashMap<GameId, NodeIndex>,
    ) -> Result<(), BracketError> {
        match source {
            TeamSource::WinnerOf(src_idx) => {
                graph.add_edge(
                    *game_to_node
                        .get(&src_idx)
                        .ok_or(BracketError::MissingGameNode(src_idx))?,
                    target,
                    EdgeType::Winner,
                );
            }
            TeamSource::LoserOf(src_idx) => {
                graph.add_edge(
                    *game_to_node
                        .get(&src_idx)
                        .ok_or(BracketError::MissingGameNode(src_idx))?,
                    target,
                    EdgeType::Loser,
                );
            }
            TeamSource::GroupOutcome(_) => {
                // No edge needed for group outcomes
            }
        }
        Ok(())
    }

    /// Resolve a team from a source (pure function)
    pub fn resolve_team<T: Tiebreaker>(
        &self,
        source: TeamSource,
        state: &BracketState,
        groups: &Groups,
        group_rules: &Rules<T>,
    ) -> Option<TeamId> {
        match source {
            TeamSource::GroupOutcome(outcome) => {
                Some(crate::playoff::transition::resolve_from_group_outcome(
                    groups,
                    &outcome,
                    group_rules,
                ))
            }
            TeamSource::WinnerOf(game_id) => state.result(game_id).map(|r| r.winner()),
            TeamSource::LoserOf(game_id) => state.result(game_id).map(|r| r.loser()),
        }
    }

    /// Get the computed state of a specific game (pure function)
    pub fn game_state<T: Tiebreaker>(
        &self,
        node_idx: NodeIndex,
        state: &BracketState,
        groups: &Groups,
        group_rules: &Rules<T>,
    ) -> PlayoffGameState {
        let game_id = self.graph[node_idx];

        // Check if already played
        if let Some(result) = state.result(game_id) {
            return PlayoffGameState::Played {
                game_id,
                result: *result,
            };
        }

        // Try to resolve teams
        let (home_source, away_source) = &self.sources[&node_idx];

        let home = self.resolve_team(home_source.clone(), state, groups, group_rules);
        let away = self.resolve_team(away_source.clone(), state, groups, group_rules);

        match (home, away) {
            (Some(h), Some(a)) => PlayoffGameState::Ready {
                game_id,
                home: h,
                away: a,
            },
            (Some(h), None) => PlayoffGameState::HomeKnown {
                game_id,
                home: h,
                away_source: away_source.clone(),
            },
            (None, Some(a)) => PlayoffGameState::AwayKnown {
                game_id,
                home_source: home_source.clone(),
                away: a,
            },
            (None, None) => PlayoffGameState::Pending {
                game_id,
                home_source: home_source.clone(),
                away_source: away_source.clone(),
            },
        }
    }

    /// Get all games and their states (pure function)
    pub fn all_game_states<T: Tiebreaker>(
        &self,
        state: &BracketState,
        groups: &Groups,
        group_rules: &Rules<T>,
    ) -> Vec<PlayoffGameState> {
        self.graph
            .node_indices()
            .map(|idx| self.game_state(idx, state, groups, group_rules))
            .collect()
    }

    /// Get games at a specific depth from the final
    ///
    /// Depth 0 = final, depth 1 = semifinals, depth 2 = quarterfinals, etc.
    pub fn games_at_depth<T: Tiebreaker>(
        &self,
        depth: usize,
        state: &BracketState,
        groups: &Groups,
        group_rules: &Rules<T>,
    ) -> Vec<PlayoffGameState> {
        use petgraph::algo::dijkstra;
        use petgraph::visit::Reversed;

        // Calculate distances from final (going backwards through edges)
        // Use Reversed to traverse incoming edges instead of outgoing
        let distances = dijkstra(Reversed(&self.graph), self.final_node, None, |_| 1);

        self.graph
            .node_indices()
            .filter(|idx| distances.get(idx) == Some(&depth))
            .map(|idx| self.game_state(idx, state, groups, group_rules))
            .collect()
    }

    /// Get the champion (if final is played)
    pub fn champion(&self, state: &BracketState) -> Option<TeamId> {
        let final_game_id = self.graph[self.final_node];
        state.result(final_game_id).map(|r| r.winner())
    }

    /// Get the runner-up (if final is played)
    pub fn runner_up(&self, state: &BracketState) -> Option<TeamId> {
        let final_game_id = self.graph[self.final_node];
        state.result(final_game_id).map(|r| r.loser())
    }

    /// Get all game IDs in the bracket
    pub fn all_game_ids(&self) -> impl Iterator<Item = GameId> + '_ {
        self.graph.node_weights().copied()
    }

    /// Get the game that depends on this game (for a specific edge type)
    pub fn dependents(&self, game_id: GameId, edge_type: EdgeType) -> Vec<GameId> {
        if let Some(&node_idx) = self.game_to_node.get(&game_id) {
            self.graph
                .edges_directed(node_idx, Direction::Outgoing)
                .filter(|edge| *edge.weight() == edge_type)
                .map(|edge| self.graph[edge.target()])
                .collect()
        } else {
            Vec::new()
        }
    }
}

/// The computed state of a playoff game
///
/// This is a view derived from `BracketStructure` + `BracketState`.
/// It represents the current status of a game from the user's perspective.
#[derive(Debug, Clone)]
pub enum PlayoffGameState {
    /// Both teams unknown - waiting for previous games
    Pending {
        game_id: GameId,
        home_source: TeamSource,
        away_source: TeamSource,
    },
    /// Only home team known
    HomeKnown {
        game_id: GameId,
        home: TeamId,
        away_source: TeamSource,
    },
    /// Only away team known
    AwayKnown {
        game_id: GameId,
        home_source: TeamSource,
        away: TeamId,
    },
    /// Both teams known, not yet played
    Ready {
        game_id: GameId,
        home: TeamId,
        away: TeamId,
    },
    /// Game completed
    Played {
        game_id: GameId,
        result: PlayoffResult,
    },
}

impl PlayoffGameState {
    pub fn game_id(&self) -> GameId {
        match self {
            Self::Pending { game_id, .. }
            | Self::HomeKnown { game_id, .. }
            | Self::AwayKnown { game_id, .. }
            | Self::Ready { game_id, .. }
            | Self::Played { game_id, .. } => *game_id,
        }
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }

    pub fn is_played(&self) -> bool {
        matches!(self, Self::Played { .. })
    }

    pub fn teams(&self) -> Option<(TeamId, TeamId)> {
        match self {
            Self::Ready { home, away, .. } => Some((*home, *away)),
            Self::Played { result, .. } => Some((result.home, result.away)),
            _ => None,
        }
    }
}

#[derive(Error, Debug, Clone, Copy)]
pub enum BracketError {
    #[error("Final game not found in template")]
    FinalGameNotFound,
    #[error("Bracket contains a cycle")]
    CyclicBracket,
    #[error("{0:?} not found in games_to_node map.")]
    MissingGameNode(GameId),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group::GroupId;
    use crate::group::mock_data::groups_and_teams;
    use crate::group::order::fifa_2018_rules;

    #[test]
    fn test_simple_bracket() {
        let (groups, _teams) = groups_and_teams();
        let rules = fifa_2018_rules();

        // Create a simple 2-game bracket
        let template = BracketTemplate {
            games: vec![
                (
                    GameId::from(1),
                    (
                        TeamSource::GroupOutcome(GroupOutcome::Winner(
                            GroupId::try_from('A').unwrap(),
                        )),
                        TeamSource::GroupOutcome(GroupOutcome::RunnerUp(
                            GroupId::try_from('B').unwrap(),
                        )),
                    ),
                ),
                (
                    GameId::from(2),
                    (
                        TeamSource::GroupOutcome(GroupOutcome::Winner(
                            GroupId::try_from('B').unwrap(),
                        )),
                        TeamSource::GroupOutcome(GroupOutcome::RunnerUp(
                            GroupId::try_from('A').unwrap(),
                        )),
                    ),
                ),
            ],
            final_game_id: GameId::from(2),
        };

        let bracket = BracketStructure::from_template(template).unwrap();
        let state = BracketState::new();

        let games = bracket.all_game_states(&state, &groups, &rules);
        assert_eq!(games.len(), 2);

        // Both games should be ready (teams from group stage)
        assert!(games.iter().all(|g| g.is_ready()));
    }
}
