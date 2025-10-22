//! Predefined bracket templates for common tournament formats

use crate::game::GameId;
use crate::playoff::bracket::TeamSource;

/// Template for defining bracket structure
pub struct BracketTemplate {
    pub games: Vec<(GameId, (TeamSource, TeamSource))>,
    pub final_game_id: GameId,
}
