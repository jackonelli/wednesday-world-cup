use crate::game::GoalCount;
use crate::team::TeamId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Score of a playoff game
///
/// Playoff games must have a winner, either from regular time or penalties.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayoffScore {
    home: GoalCount,
    away: GoalCount,
    home_penalty: Option<GoalCount>,
    away_penalty: Option<GoalCount>,
}

impl PlayoffScore {
    /// Create a playoff score with validation
    ///
    /// # Errors
    ///
    /// - If the game is a draw and no penalty scores are provided
    /// - If the game has a winner in regular time but penalties are provided
    /// - If only one team has a penalty score
    pub fn try_new(
        home: GoalCount,
        away: GoalCount,
        home_penalty: Option<GoalCount>,
        away_penalty: Option<GoalCount>,
    ) -> Result<Self, PlayoffError> {
        if home == away {
            // Draw in regular time - need penalties
            match (home_penalty, away_penalty) {
                (None, None) => Err(PlayoffError::NoWinner),
                (Some(_), None) => Err(PlayoffError::SinglePenalty),
                (None, Some(_)) => Err(PlayoffError::SinglePenalty),
                (Some(hp), Some(ap)) if hp == ap => Err(PlayoffError::NoWinner),
                (Some(_), Some(_)) => Ok(PlayoffScore {
                    home,
                    away,
                    home_penalty,
                    away_penalty,
                }),
            }
        } else {
            // Winner in regular time - no penalties allowed
            match (home_penalty, away_penalty) {
                (None, None) => Ok(PlayoffScore {
                    home,
                    away,
                    home_penalty,
                    away_penalty,
                }),
                _ => Err(PlayoffError::WinnerAndPenalty),
            }
        }
    }

    /// Create score from regular time result (no penalties)
    pub fn regular_time(home: GoalCount, away: GoalCount) -> Result<Self, PlayoffError> {
        Self::try_new(home, away, None, None)
    }

    /// Create score with penalties
    pub fn with_penalties(
        home: GoalCount,
        away: GoalCount,
        home_penalty: GoalCount,
        away_penalty: GoalCount,
    ) -> Result<Self, PlayoffError> {
        Self::try_new(home, away, Some(home_penalty), Some(away_penalty))
    }

    /// Get the winner of this game
    pub fn winner(&self, home_team: TeamId, away_team: TeamId) -> TeamId {
        if self.home > self.away {
            home_team
        } else if self.away > self.home {
            away_team
        } else {
            // Draw in regular time, check penalties
            match (self.home_penalty, self.away_penalty) {
                (Some(hp), Some(ap)) if hp > ap => home_team,
                (Some(_), Some(_)) => away_team,
                _ => unreachable!("PlayoffScore invariant violated: draw without valid penalties"),
            }
        }
    }

    /// Get the loser of this game
    pub fn loser(&self, home_team: TeamId, away_team: TeamId) -> TeamId {
        if self.winner(home_team, away_team) == home_team {
            away_team
        } else {
            home_team
        }
    }

    /// Check if game went to penalties
    pub fn went_to_penalties(&self) -> bool {
        self.home_penalty.is_some() || self.away_penalty.is_some()
    }

    /// Get regular time scores
    pub fn regular_time_score(&self) -> (GoalCount, GoalCount) {
        (self.home, self.away)
    }

    /// Get penalty scores (if any)
    pub fn penalty_score(&self) -> Option<(GoalCount, GoalCount)> {
        match (self.home_penalty, self.away_penalty) {
            (Some(h), Some(a)) => Some((h, a)),
            _ => None,
        }
    }
}

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlayoffError {
    #[error("No winner in playoff game")]
    NoWinner,
    #[error("Winner in regular time but penalty scores provided")]
    WinnerAndPenalty,
    #[error("Only one team has a penalty score")]
    SinglePenalty,
    #[error("Game teams not yet known")]
    TeamsNotKnown,
    #[error("Game not ready to be played")]
    GameNotReady,
    #[error("Playoff transition id's not a subset of group id's")]
    TransitionGroupIdMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regular_time_winner() {
        let score = PlayoffScore::regular_time(
            GoalCount::try_from(2).unwrap(),
            GoalCount::try_from(1).unwrap(),
        )
        .unwrap();
        assert!(!score.went_to_penalties());
        assert_eq!(
            score.regular_time_score(),
            (
                GoalCount::try_from(2).unwrap(),
                GoalCount::try_from(1).unwrap()
            )
        );
    }

    #[test]
    fn test_penalties() {
        let score = PlayoffScore::with_penalties(
            GoalCount::try_from(1).unwrap(),
            GoalCount::try_from(1).unwrap(),
            GoalCount::try_from(5).unwrap(),
            GoalCount::try_from(4).unwrap(),
        )
        .unwrap();
        assert!(score.went_to_penalties());
        assert_eq!(
            score.penalty_score(),
            Some((
                GoalCount::try_from(5).unwrap(),
                GoalCount::try_from(4).unwrap()
            ))
        );
    }

    #[test]
    fn test_draw_without_penalties_fails() {
        let result = PlayoffScore::regular_time(
            GoalCount::try_from(1).unwrap(),
            GoalCount::try_from(1).unwrap(),
        );
        assert!(matches!(result, Err(PlayoffError::NoWinner)));
    }

    #[test]
    fn test_winner_with_penalties_fails() {
        let result = PlayoffScore::with_penalties(
            GoalCount::try_from(2).unwrap(),
            GoalCount::try_from(1).unwrap(),
            GoalCount::try_from(5).unwrap(),
            GoalCount::try_from(4).unwrap(),
        );
        assert!(matches!(result, Err(PlayoffError::WinnerAndPenalty)));
    }

    #[test]
    fn test_single_penalty_fails() {
        let result = PlayoffScore::try_new(
            GoalCount::try_from(1).unwrap(),
            GoalCount::try_from(1).unwrap(),
            Some(GoalCount::try_from(5).unwrap()),
            None,
        );
        assert!(matches!(result, Err(PlayoffError::SinglePenalty)));
    }
}
