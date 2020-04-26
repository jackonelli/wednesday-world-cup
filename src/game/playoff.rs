use crate::game::{Game, GoalCount};
use crate::team::TeamId;
use crate::Date;
use thiserror::Error;

#[derive(Clone)]
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

pub struct PrePlayoffGame {
    home: TeamId,
    away: TeamId,
    date: Date,
}

impl PrePlayoffGame {
    pub fn new(home: TeamId, away: TeamId, date: Date) -> Self {
        Self { home, away, date }
    }
}

impl Game for PrePlayoffGame {
    fn home_team(&self) -> TeamId {
        self.home
    }
    fn away_team(&self) -> TeamId {
        self.away
    }
}

#[derive(Error, Debug)]
pub enum PlayoffError {
    #[error("No winner in playoff game")]
    NoWinner,
    #[error("Winner score and penalty")]
    WinnerAndPenalty,
}

#[cfg(test)]
mod tests {}
