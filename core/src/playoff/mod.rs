//! Tournament playoff
pub mod game;
pub mod transition;
use self::game::PlayoffGame;
use thiserror::Error;

use crate::game::GameId;
use std::collections::HashMap;

struct Playoff {
    rounds: HashMap<RoundIdx, Round>,
}

struct Round {
    games: HashMap<GameId, PlayoffGame>,
}

struct RoundIdx(u8);

#[cfg(test)]
mod tests {
    #[test]
    fn simple_setup() {}
}

#[derive(Error, Debug)]
pub enum PlayoffError {
    #[error("Playoff transition id's not a subset of group id's")]
    TransitionGroupIdMismatch,
}
