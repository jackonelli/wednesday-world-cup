//! Tournament playoff
mod game;
use self::game::PlayoffGame;
use crate::game::GameId;
use std::collections::HashMap;

struct Playoff {
    rounds: Vec<Round>,
}

struct Round {
    games: HashMap<GameId, PlayoffGame>,
}
