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
/// TODO: Cap it to an insane but limited number so that casts to i32 are safe.
/// This value is essentially the only thing the user can interact with, by predicting scores in the ui.
/// In aggragate the thing we're risking is surpassing i32::MAX for like one teams total goal count in the group.
/// Or later perhaps the total number scored by a team in a tournament.
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
