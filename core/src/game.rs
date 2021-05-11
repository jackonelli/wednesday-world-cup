//! General tournament game
//!
//! Specification and implementation in this module is strictly limited compared to what one would
//! expect from data structures describing a game.
//! More concrete implementations are found in the group and playoff modules respectively.
use crate::team::TeamId;
use derive_more::{Add, AddAssign, Display, From, Into, Neg};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ops::Sub;
use std::str::FromStr;
use thiserror::Error;

/// General game score.
///
/// Accepts any pair of non-negative integers
/// Score associated with [`PlayedGroupGame`]
///
/// Determines the outcome of a game which can be, win, loss or draw.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Eq, PartialEq)]
pub struct Score {
    pub home: GoalCount,
    pub away: GoalCount,
}

impl Score {
    pub fn new<T: Into<GoalCount>>(home_goals: T, away_goals: T) -> Self {
        Score {
            home: home_goals.into(),
            away: away_goals.into(),
        }
    }
    pub fn home_outcome(&self) -> Outcome {
        match self.home.cmp(&self.away) {
            Ordering::Greater => Outcome::Win,
            Ordering::Less => Outcome::Lose,
            Ordering::Equal => Outcome::Draw,
        }
    }
    pub fn away_outcome(&self) -> Outcome {
        match self.home_outcome() {
            Outcome::Win => Outcome::Lose,
            Outcome::Draw => Outcome::Draw,
            Outcome::Lose => Outcome::Win,
        }
    }
}

impl<T: Into<GoalCount>> From<(T, T)> for Score {
    fn from(x: (T, T)) -> Self {
        let (home, away) = x;
        Self {
            home: home.into(),
            away: away.into(),
        }
    }
}

// TODO test.
impl FromStr for Score {
    type Err = GameError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let score_split: Vec<&str> = s.split('-').collect();
        let (home, away) = if score_split.len() != 2 {
            return Err(GameError::ScoreParse(String::from(s)));
        } else {
            (score_split[0], score_split[1])
        };
        //TODO: Better error handling
        let home = home
            .parse::<u32>()
            .map_err(|_err| GameError::ScoreParse(String::from(s)))?;
        let away = away
            .parse::<u32>()
            .map_err(|_err| GameError::ScoreParse(String::from(s)))?;
        Ok(Score::from((home, away)))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, Eq, PartialEq)]
pub enum Outcome {
    Win,
    Draw,
    Lose,
}

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

pub trait Game {
    fn home_team(&self) -> TeamId;
    fn away_team(&self) -> TeamId;
}

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Error parsing score: '{0}'")]
    ScoreParse(String),
}
