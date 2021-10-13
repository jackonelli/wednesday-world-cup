//! # General tournament game
//!
//! Specification and implementation in this module is strictly limited, compared to what one would
//! expect from data structures describing a game.
//! More concrete implementations are found in the group and playoff modules respectively.
use crate::team::TeamId;
use derive_more::{Add, AddAssign, Display, From, Into, Neg};
use serde::{Deserialize, Serialize};
use std::ops::Sub;

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
)]
pub struct GoalCount(u32);

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
