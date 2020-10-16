//! Fair play scoring
use derive_more::{Add, AddAssign, Display, From};
use serde::{Deserialize, Serialize};
use std::ops::Mul;

/// Fair play data
///
/// Currently based on the Fifa World Cup 2018 Rules:
///
/// - Yellow card: -1 points;
/// - Indirect red card (second yellow card): -3 points;
/// - Direct red card: -4 points;
/// - Yellow card and direct red card: -5 points;
///
/// ```
/// # use wwc_core::fair_play::{FairPlay, FifaFairPlayValue, FairPlayValue};
/// let fair_play = FairPlay::new(1, 2, 3, 4);
/// assert_eq!(FifaFairPlayValue::from(39), FifaFairPlayValue::from_fair_play(&fair_play));
/// ```
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FairPlay {
    yellow: CardCount,
    indirect_red: CardCount,
    direct_red: CardCount,
    yellow_and_direct: CardCount,
}

impl FairPlay {
    pub fn new<C: Into<CardCount>>(
        yellow: C,
        indirect_red: C,
        direct_red: C,
        yellow_and_direct: C,
    ) -> Self {
        FairPlay {
            yellow: yellow.into(),
            indirect_red: indirect_red.into(),
            direct_red: direct_red.into(),
            yellow_and_direct: yellow_and_direct.into(),
        }
    }
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug, Default)]
pub struct FairPlayScore {
    pub home: FairPlay,
    pub away: FairPlay,
}

impl FairPlayScore {
    pub fn new(home: FairPlay, away: FairPlay) -> Self {
        Self { home, away }
    }
}

pub trait FairPlayValue {
    fn from_fair_play(fp: &FairPlay) -> Self;
}

#[derive(
    Copy,
    Clone,
    Display,
    Debug,
    Serialize,
    Deserialize,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Add,
    AddAssign,
)]
pub struct FifaFairPlayValue(i8);

impl FairPlayValue for FifaFairPlayValue {
    /// Calculate fair play value based on Fifa rules.
    fn from_fair_play(fp: &FairPlay) -> Self {
        Self(fp.yellow * -1 + fp.indirect_red * -3 + fp.direct_red * -4 + fp.yellow_and_direct * -5)
    }
}

// TODO: This trait impl is good for internal (test) ergonomics,
// but I would rather not leak it to the pub API.
// Private trait impl possible?
impl From<u8> for FifaFairPlayValue {
    fn from(magnitude: u8) -> Self {
        FifaFairPlayValue(-(magnitude as i8))
    }
}

impl num::Zero for FifaFairPlayValue {
    fn zero() -> Self {
        FifaFairPlayValue(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Add,
    AddAssign,
)]
pub struct UefaFairPlayValue(i8);

impl FairPlayValue for UefaFairPlayValue {
    /// Calculate fair play value based on Fifa rules.
    fn from_fair_play(fp: &FairPlay) -> Self {
        Self(fp.yellow * -1 + fp.indirect_red * -3 + fp.direct_red * -3 + fp.yellow_and_direct * -5)
    }
}

// TODO: This trait impl is good for internal (test) ergonomics,
// but I would rather not leak it to the pub API.
// Private trait impl possible?
impl From<u8> for UefaFairPlayValue {
    fn from(magnitude: u8) -> Self {
        UefaFairPlayValue(-(magnitude as i8))
    }
}

impl num::Zero for UefaFairPlayValue {
    fn zero() -> Self {
        UefaFairPlayValue(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

#[derive(
    Debug, Copy, Clone, Default, Serialize, Deserialize, Eq, PartialEq, From, Add, AddAssign,
)]
pub struct CardCount(u8);

impl Mul<i8> for CardCount {
    type Output = i8;
    fn mul(self, rhs: i8) -> Self::Output {
        self.0 as i8 * rhs
    }
}
