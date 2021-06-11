//! Tournament playoff
mod game;
use self::game::PlayoffGame;
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
    fn simple_setup() {
    }
}
