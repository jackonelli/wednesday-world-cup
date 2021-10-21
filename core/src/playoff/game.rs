use crate::game::GameId;
use crate::game::GoalCount;
use crate::team::TeamId;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct PlayoffGame {
    game_id: GameId,
    home: Option<TeamId>,
    away: Option<TeamId>,
    score: Option<PlayoffScore>,
}

impl PlayoffGame {
    pub fn new(game_id: GameId, home: TeamId, away: TeamId) -> Self {
        Self {
            game_id,
            home: Some(home),
            away: Some(away),
            score: None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PlayoffScore {
    home: GoalCount,
    away: GoalCount,
    home_penalty: Option<GoalCount>,
    away_penalty: Option<GoalCount>,
}

impl PlayoffScore {
    pub fn try_new(
        home: GoalCount,
        away: GoalCount,
        home_penalty: Option<GoalCount>,
        away_penalty: Option<GoalCount>,
    ) -> Result<Self, PlayoffError> {
        if home == away {
            match (home_penalty, away_penalty) {
                (None, None) => Err(PlayoffError::NoWinner),
                (Some(_), None) => Err(PlayoffError::SinglePenalty),
                (None, Some(_)) => Err(PlayoffError::SinglePenalty),
                (Some(_), Some(_)) => Ok(PlayoffScore {
                    home,
                    away,
                    home_penalty,
                    away_penalty,
                }),
            }
        } else {
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
}

#[derive(Error, Debug, Copy, Clone)]
pub enum PlayoffError {
    #[error("No winner in playoff game")]
    NoWinner,
    #[error("Winner score and penalty")]
    WinnerAndPenalty,
    #[error("Only one team has a penalty score.")]
    SinglePenalty,
}

#[cfg(test)]
mod tests {}
