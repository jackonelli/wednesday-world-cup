//! JSON data interface
//!
//! The plan is to eventually move the data module to a separate crate.
//!
//! Data source: https://github.com/lsv/fifa-worldcup-2018
use crate::game::GoalCount;
use crate::group::GroupId;
use crate::team::{Team, TeamId};
use crate::Date;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Data {
    pub teams: Vec<Team>,
    pub groups: HashMap<GroupId, ParseGroup>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ParseGroup {
    name: String,
    #[serde(rename = "matches")]
    games: Vec<ParseGame>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ParseGame {
    #[serde(rename = "name")]
    id: u8,
    #[serde(rename = "type")]
    type_: GameType,
    home_team: TeamId,
    away_team: TeamId,
    home_result: GoalCount,
    away_result: GoalCount,
    home_penalty: Option<GoalCount>,
    away_penalty: Option<GoalCount>,
    finished: bool,
    date: Date,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum GameType {
    Group,
    Qualified,
    Winner,
    Loser,
}
