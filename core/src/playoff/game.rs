use crate::game::GameId;
use crate::game::GoalCount;
use crate::team::TeamId;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct PlayoffGame {
    game_id: GameId,
    home: Option<TeamId>,
    away: Option<TeamId>,
    score: Option<Score>,
}

#[derive(Debug, Clone, Copy)]
pub struct PlayoffScore(GoalCount, GoalCount);

impl PlayoffScore {
    pub fn try_new(home: GoalCount, away: GoalCount) -> Result<Self, PlayoffError> {
        if home == away {
            Err(PlayoffError::NoWinner)
        } else {
            Ok(PlayoffScore(home, away))
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Score {
    pub home: GoalCount,
    pub away: GoalCount,
    pub home_penalty: Option<GoalCount>,
    pub away_penalty: Option<GoalCount>,
}

impl Score {
    pub fn try_new(
        home: GoalCount,
        away: GoalCount,
        home_penalty: Option<GoalCount>,
        away_penalty: Option<GoalCount>,
    ) -> Result<Self, PlayoffError> {
        if home == away {
            match (home_penalty, away_penalty) {
                (None, None) => Err(PlayoffError::NoWinner),
                _ => todo!(),
            }
        } else {
            match (home_penalty, away_penalty) {
                (Some(_), _) => Err(PlayoffError::WinnerAndPenalty),
                (_, Some(_)) => Err(PlayoffError::WinnerAndPenalty),
                _ => todo!(),
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
}

#[cfg(test)]
mod tests {}
