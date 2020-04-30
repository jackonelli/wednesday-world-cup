//! LSV JSON data interface
//!
//! The plan is to eventually move the data module to a separate crate.
//!
//! Data source: https://github.com/lsv/fifa-worldcup-2018
use crate::game::GoalCount;
use crate::group::game::{PlayedGroupGame, PreGroupGame};
use crate::group::{Group, GroupError, GroupId};
use crate::team::{Team, TeamId};
use crate::Date;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;

pub fn try_groups_from_data(data: &Data) -> Result<HashMap<GroupId, Group>, GroupError> {
    let groups_with_err = data.groups.iter().map(|(id, group)| {
        let group: Result<Group, GroupError> = (group.clone()).try_into();
        (id, group)
    });
    if groups_with_err.clone().any(|(_, group)| group.is_err()) {
        Err(GroupError::GenericError)
    } else {
        Ok(groups_with_err
            .map(|(id, group)| (*id, group.unwrap()))
            .collect())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Data {
    pub teams: Vec<Team>,
    pub groups: HashMap<GroupId, ParseGroup>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParseGroup {
    name: String,
    #[serde(rename = "matches")]
    games: Vec<ParseGame>,
}

impl TryInto<Group> for ParseGroup {
    type Error = GroupError;
    fn try_into(self) -> Result<Group, Self::Error> {
        let upcoming_games = self
            .games
            .iter()
            .filter(|game| !game.finished)
            .map(|game| {
                let game = *game;
                game.try_into()
            })
            .collect::<Result<Vec<PreGroupGame>, GroupError>>()?;
        let played_games = self
            .games
            .iter()
            .filter(|game| game.finished)
            .map(|game| {
                let game = *game;
                game.try_into()
            })
            .collect::<Result<Vec<PlayedGroupGame>, GroupError>>()?;
        Group::try_new(upcoming_games, played_games)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
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

impl TryInto<PreGroupGame> for ParseGame {
    type Error = GroupError;
    fn try_into(self) -> Result<PreGroupGame, Self::Error> {
        PreGroupGame::try_new(self.id, self.home_team, self.away_team, self.date)
    }
}

impl TryInto<PlayedGroupGame> for ParseGame {
    type Error = GroupError;
    fn try_into(self) -> Result<PlayedGroupGame, Self::Error> {
        PlayedGroupGame::try_new(
            self.id,
            self.home_team,
            self.away_team,
            (self.home_result, self.away_result),
            (0, 0),
            self.date,
        )
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum GameType {
    Group,
    Qualified,
    Winner,
    Loser,
}
