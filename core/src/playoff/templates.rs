//! Predefined bracket templates for common tournament formats

use crate::game::GameId;
use crate::group::{GroupId, GroupOutcome};
use crate::playoff::bracket::{BracketTemplate, TeamSource};
use petgraph::graph::NodeIndex;

/// Create an 8-team single elimination bracket
///
/// Structure:
/// ```text
///   QF1 ─┐
///        ├─ SF1 ─┐
///   QF2 ─┘       │
///                ├─ Final
///   QF3 ─┐       │
///        ├─ SF2 ─┘
///   QF4 ─┘
/// ```
pub fn single_elimination_8() -> BracketTemplate {
    let mut games = Vec::new();

    // Quarter-finals (games 1-4)
    for i in 0u32..4 {
        let group_a = GroupId::try_from((b'A' + i as u8) as char).unwrap();
        let group_b = GroupId::try_from((b'A' + ((i + 1) % 4) as u8) as char).unwrap();
        println!("Game {}: {} vs {}", i, group_a, group_b);
        games.push((
            GameId::from(i + 1),
            (
                TeamSource::GroupOutcome(GroupOutcome::Winner(group_a)),
                TeamSource::GroupOutcome(GroupOutcome::RunnerUp(group_b)),
            ),
        ));
    }

    // Semi-finals (games 5-6)
    games.push((
        GameId::from(5),
        (
            TeamSource::WinnerOf(GameId::from(1)), // Winner of QF1
            TeamSource::WinnerOf(GameId::from(2)), // Winner of QF2
        ),
    ));
    games.push((
        GameId::from(6),
        (
            TeamSource::WinnerOf(GameId::from(3)), // Winner of QF3
            TeamSource::WinnerOf(GameId::from(4)), // Winner of QF4
        ),
    ));

    // Final (game 7)
    games.push((
        GameId::from(7),
        (
            TeamSource::WinnerOf(GameId::from(5)), // Winner of SF1
            TeamSource::WinnerOf(GameId::from(6)), // Winner of SF2
        ),
    ));

    BracketTemplate {
        games,
        final_game_id: GameId::from(7),
    }
}

/// Euro 2020 knockout bracket (simplified version)
///
/// This is a simplified version. Full Euro 2020 has 16 teams with
/// complex third-place qualification rules.
///
/// For a complete implementation, you would need to model:
/// - Best third-place teams from 6 groups
/// - Specific matchup rules based on group positions
pub fn euro_2020_simplified() -> BracketTemplate {
    // This would be fully implemented with all 16 teams
    // For now, returning the 8-team version as a placeholder
    single_elimination_8()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playoff::bracket::BracketStructure;

    #[test]
    fn test_single_elimination_8_valid() {
        let template = single_elimination_8();
        let result = BracketStructure::from_template(template);
        assert!(result.is_ok());
    }
}
