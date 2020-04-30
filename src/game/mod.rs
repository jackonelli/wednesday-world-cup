use crate::group::stats::Unary;
use crate::team::TeamId;
use derive_more::{Add, AddAssign, From, Neg};
use serde::{Deserialize, Serialize};
use std::ops::Sub;
pub mod group;
pub mod playoff;

#[derive(
    Default,
    Debug,
    Deserialize,
    Serialize,
    Clone,
    Copy,
    From,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Add,
    AddAssign,
)]
pub struct GoalCount(pub u8);

impl Sub for GoalCount {
    type Output = GoalDiff;
    fn sub(self, other: Self) -> Self::Output {
        GoalDiff(self.0 as i8 - other.0 as i8)
    }
}

impl Unary for GoalCount {}

impl num::Zero for GoalCount {
    fn zero() -> GoalCount {
        GoalCount(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Default, Debug, Clone, Copy, From, Eq, Neg, PartialEq, Ord, PartialOrd, Add, AddAssign)]
pub struct GoalDiff(pub i8);

impl Unary for GoalDiff {}

impl num::Zero for GoalDiff {
    fn zero() -> GoalDiff {
        GoalDiff(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Default, Debug, Clone, Copy, From, Eq, PartialEq, Ord, PartialOrd, Add, AddAssign)]
pub struct NumGames(pub u8);

pub trait Game {
    fn home_team(&self) -> TeamId;
    fn away_team(&self) -> TeamId;
}
