use serde::{Deserialize, Serialize};
pub mod data;
pub mod fair_play;
pub mod game;
pub mod group;
pub mod playoff;
pub mod team;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Date {}
