use crate::team::TeamId;
use derive_more::{Add, AddAssign, From, Neg};
use std::ops::Sub;
pub mod group;
pub mod playoff;

#[derive(Default, Debug, Clone, Copy, From, Eq, PartialEq, Ord, PartialOrd, Add, AddAssign)]
pub struct GoalCount(pub u8);

impl Sub for GoalCount {
    type Output = GoalDiff;
    fn sub(self, other: Self) -> Self::Output {
        GoalDiff(self.0 as i8 - other.0 as i8)
    }
}

#[derive(Default, Debug, Clone, Copy, From, Eq, Neg, PartialEq, Ord, PartialOrd, Add, AddAssign)]
pub struct GoalDiff(pub i8);

#[derive(Default, Debug, Clone, Copy, From, Eq, PartialEq, Ord, PartialOrd, Add, AddAssign)]
pub struct NumGames(pub u8);

pub trait Game {
    fn home_team(&self) -> TeamId;
    fn away_team(&self) -> TeamId;
}
