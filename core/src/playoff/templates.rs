//! Predefined bracket templates for common tournament formats

use crate::game::GameId;
use crate::group::{GroupId, GroupOutcome};
use crate::playoff::bracket::{BracketTemplate, TeamSource};
use petgraph::graph::NodeIndex;

/// Create a simple 4-team single elimination bracket
///
/// Structure:
/// ```text
///   Semi 1 ─┐
///           ├─ Final
///   Semi 2 ─┘
/// ```
pub fn simple_four_team() -> BracketTemplate {
    BracketTemplate {
        games: vec![
            // Semi-final 1
            (
                GameId::from(1),
                (
                    TeamSource::GroupOutcome(GroupOutcome::Winner(GroupId::try_from('A').unwrap())),
                    TeamSource::GroupOutcome(GroupOutcome::RunnerUp(
                        GroupId::try_from('B').unwrap(),
                    )),
                ),
            ),
            // Semi-final 2
            (
                GameId::from(2),
                (
                    TeamSource::GroupOutcome(GroupOutcome::Winner(GroupId::try_from('B').unwrap())),
                    TeamSource::GroupOutcome(GroupOutcome::RunnerUp(
                        GroupId::try_from('A').unwrap(),
                    )),
                ),
            ),
            // Final
            (
                GameId::from(3),
                (
                    TeamSource::WinnerOf(NodeIndex::new(0)), // Winner of game 1
                    TeamSource::WinnerOf(NodeIndex::new(1)), // Winner of game 2
                ),
            ),
        ],
        final_game_id: GameId::from(3),
    }
}

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
            TeamSource::WinnerOf(NodeIndex::new(0)), // Winner of QF1
            TeamSource::WinnerOf(NodeIndex::new(1)), // Winner of QF2
        ),
    ));
    games.push((
        GameId::from(6),
        (
            TeamSource::WinnerOf(NodeIndex::new(2)), // Winner of QF3
            TeamSource::WinnerOf(NodeIndex::new(3)), // Winner of QF4
        ),
    ));

    // Final (game 7)
    games.push((
        GameId::from(7),
        (
            TeamSource::WinnerOf(NodeIndex::new(4)), // Winner of SF1
            TeamSource::WinnerOf(NodeIndex::new(5)), // Winner of SF2
        ),
    ));

    BracketTemplate {
        games,
        final_game_id: GameId::from(7),
    }
}

/// Create a bracket with third-place playoff
///
/// Structure:
/// ```text
///   Semi 1 ─┐
///           ├─ Final
///   Semi 2 ─┘
///
///   (Semi 1 loser) ─┐
///                   ├─ 3rd Place
///   (Semi 2 loser) ─┘
/// ```
pub fn with_third_place_playoff() -> BracketTemplate {
    BracketTemplate {
        games: vec![
            // Semi-final 1
            (
                GameId::from(1),
                (
                    TeamSource::GroupOutcome(GroupOutcome::Winner(GroupId::try_from('A').unwrap())),
                    TeamSource::GroupOutcome(GroupOutcome::RunnerUp(
                        GroupId::try_from('B').unwrap(),
                    )),
                ),
            ),
            // Semi-final 2
            (
                GameId::from(2),
                (
                    TeamSource::GroupOutcome(GroupOutcome::Winner(GroupId::try_from('B').unwrap())),
                    TeamSource::GroupOutcome(GroupOutcome::RunnerUp(
                        GroupId::try_from('A').unwrap(),
                    )),
                ),
            ),
            // Third-place playoff
            (
                GameId::from(3),
                (
                    TeamSource::LoserOf(NodeIndex::new(0)), // Loser of semi 1
                    TeamSource::LoserOf(NodeIndex::new(1)), // Loser of semi 2
                ),
            ),
            // Final
            (
                GameId::from(4),
                (
                    TeamSource::WinnerOf(NodeIndex::new(0)), // Winner of semi 1
                    TeamSource::WinnerOf(NodeIndex::new(1)), // Winner of semi 2
                ),
            ),
        ],
        final_game_id: GameId::from(4),
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
    fn test_simple_four_team_valid() {
        let template = simple_four_team();
        let result = BracketStructure::from_template(template);
        assert!(result.is_ok());
    }

    #[test]
    fn test_single_elimination_8_valid() {
        let template = single_elimination_8();
        let result = BracketStructure::from_template(template);
        assert!(result.is_ok());
    }

    #[test]
    fn test_third_place_playoff_valid() {
        let template = with_third_place_playoff();
        let result = BracketStructure::from_template(template);
        assert!(result.is_ok());
    }
}
