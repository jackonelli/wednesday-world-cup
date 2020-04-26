use derive_more::{Add, AddAssign, From};
use std::ops::Mul;
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
    pub(crate) fn new<C: Into<CardCount>>(
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
    pub fn score(&self) -> FairPlayScore {
        self.yellow * -1
            + self.indirect_red * -3
            + self.direct_red * -4
            + self.yellow_and_direct * -5
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, From, Add, AddAssign)]
pub struct FairPlayScore(i8);

impl FairPlayScore {
    pub(crate) fn new(val: u8) -> Self {
        FairPlayScore(-(val as i8))
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, From, Add, AddAssign)]
pub struct CardCount(u8);

impl<T> Mul<T> for CardCount
where
    T: Into<i8>,
{
    type Output = FairPlayScore;
    fn mul(self, rhs: T) -> Self::Output {
        FairPlayScore(self.0 as i8 * rhs.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn score() {
        let fair_play = FairPlay::new(1, 2, 3, 4);
        assert_eq!(-39, fair_play.score().0);
    }
}
