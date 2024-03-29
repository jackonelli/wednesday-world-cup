//! Player/User/Better/Predictor
use crate::game::GameId;
use crate::group::game::{GroupGameScore, PlayedGroupGame};
use derive_more::{Display, From, Into};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    name: String,
    id: PlayerId,
}

impl Player {
    pub fn dummy() -> Self {
        Player {
            name: String::from("Dummy"),
            id: PlayerId(1),
        }
    }

    pub fn id(&self) -> PlayerId {
        self.id
    }
}

/// Numeric player id for db.
#[derive(Display, Debug, Copy, Clone, From, Into, Serialize, Deserialize)]
pub struct PlayerId(i32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPredictions {
    pub id: PlayerId,
    preds: Vec<Prediction>,
}

impl PlayerPredictions {
    pub fn new(player_id: PlayerId, preds: Vec<Prediction>) -> Self {
        PlayerPredictions {
            id: player_id,
            preds,
        }
    }
}

impl PlayerPredictions {
    pub fn preds(&self) -> impl Iterator<Item = &Prediction> {
        self.preds.iter()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Prediction(pub GameId, pub GroupGameScore);

impl std::fmt::Display for Prediction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Game {}: {}", self.0, self.1)
    }
}

impl From<PlayedGroupGame> for Prediction {
    fn from(game: PlayedGroupGame) -> Self {
        Prediction(game.id, game.score)
    }
}
