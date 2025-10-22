//! # General tournament game
//!
//! Specification and implementation in this module is strictly limited, compared to what one would
//! expect from data structures describing a game.
//! More concrete implementations are found in the group and playoff modules respectively.
use crate::team::TeamId;
use derive_more::{Add, AddAssign, Display, From, Into, Neg, Sum};
use serde::{Deserialize, Serialize};
use std::ops::Sub;

/// Common game functionality
pub trait Game {
    fn home_team(&self) -> TeamId;
    fn away_team(&self) -> TeamId;
}

/// Internal unique identifier for games.
///
/// This is common for all games througout the application.
#[derive(
    Debug,
    Display,
    Deserialize,
    Serialize,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
    From,
    Into,
)]
pub struct GameId(u32);

/// Non-negative int. of goals.
///
/// Either in a single game or aggregated, like in number of goals scored in a group stage.
///
/// Capped at 999 to ensure safe casts to i32 and prevent overflow issues.
/// This value is essentially the only thing the user can interact with, by predicting scores in the ui.
#[derive(
    Default,
    Debug,
    Display,
    Deserialize,
    Serialize,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Add,
    AddAssign,
    Sum,
)]
pub struct GoalCount(u32);

impl GoalCount {
    pub const MAX: u32 = 999;

    /// Create a new GoalCount from a u32, capped at MAX
    pub fn new(value: u32) -> Result<Self, GoalCountError> {
        if value <= Self::MAX {
            Ok(GoalCount(value))
        } else {
            Err(GoalCountError::Overflow(value))
        }
    }

    /// Get the inner value as u32
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for GoalCount {
    type Error = GoalCountError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<GoalCount> for u32 {
    fn from(count: GoalCount) -> u32 {
        count.0
    }
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum GoalCountError {
    #[display("Goal count {_0} exceeds maximum of {}", GoalCount::MAX)]
    Overflow(u32),
}

impl Sub for GoalCount {
    type Output = GoalDiff;
    fn sub(self, other: Self) -> Self::Output {
        GoalDiff(self.0 as i32 - other.0 as i32)
    }
}

impl num::Zero for GoalCount {
    fn zero() -> GoalCount {
        GoalCount(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

/// Integer goal difference.
///
/// Number of goals scored minus number of goals conceded.
/// Either in a single game or aggregated, e.g. in a group stage.
#[derive(
    Default,
    Debug,
    Display,
    Deserialize,
    Serialize,
    Clone,
    Copy,
    From,
    Eq,
    Neg,
    PartialEq,
    Ord,
    PartialOrd,
    Add,
    AddAssign,
    Sum,
)]
pub struct GoalDiff(pub i32);

impl num::Zero for GoalDiff {
    fn zero() -> GoalDiff {
        GoalDiff(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

/// Non-negative int. games count.
///
/// Used for example in group ordering with number of wins/draws/losses.
#[derive(
    Default,
    Debug,
    Display,
    Deserialize,
    Serialize,
    Clone,
    Copy,
    From,
    Into,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Add,
    AddAssign,
    Sum,
)]
pub struct NumGames(pub u32);

impl num::Zero for NumGames {
    fn zero() -> NumGames {
        NumGames(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}
