use derive_more::{Add, AddAssign, From};
use serde::{Deserialize, Serialize};
use std::ops::Mul;

/// Fifa World Cup 2018 Rules:
///
///Yellow card: –1 points;
///Indirect red card (second yellow card): –3 points;
///Direct red card: –4 points;
///Yellow card and direct red card: –5 points;
#[derive(Default, Debug, Clone)]
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
    pub fn value(&self) -> FairPlayValue {
        self.yellow * -1
            + self.indirect_red * -3
            + self.direct_red * -4
            + self.yellow_and_direct * -5
    }
}

#[derive(Copy, Clone, Deserialize, Serialize, Debug, Default, Eq, PartialEq, Add, AddAssign)]
pub struct FairPlayScore {
    pub home: FairPlayValue,
    pub away: FairPlayValue,
}

impl<T: Into<FairPlayValue>> From<(T, T)> for FairPlayScore {
    fn from(x: (T, T)) -> Self {
        Self {
            home: x.0.into(),
            away: x.1.into(),
        }
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
pub struct FairPlayValue(i8);

impl From<u8> for FairPlayValue {
    fn from(magnitude: u8) -> Self {
        FairPlayValue(-(magnitude as i8))
    }
}

impl num::Zero for FairPlayValue {
    fn zero() -> Self {
        FairPlayValue(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, From, Add, AddAssign)]
pub struct CardCount(u8);

impl<T> Mul<T> for CardCount
where
    T: Into<i8>,
{
    type Output = FairPlayValue;
    fn mul(self, rhs: T) -> Self::Output {
        FairPlayValue(self.0 as i8 * rhs.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    /// TODO: Have as a doc test instead.
    fn score() {
        let fair_play = FairPlay::new(1, 2, 3, 4);
        assert_eq!(-39, fair_play.value().0);
    }
}
