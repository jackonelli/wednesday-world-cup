use derive_more::{Add, AddAssign};
pub mod group;
pub mod playoff;

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Add, AddAssign)]
pub struct GoalCount(pub u8);

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Add, AddAssign)]
pub struct NumGames(pub u8);
