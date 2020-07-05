//! LSV JSON data interface
//!
//! The plan is to eventually move the data module to a separate crate.
//!
//! Data source: https://github.com/lsv/fifa-worldcup-2018
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use thiserror::Error;
use wwc_core::game::GoalCount;
use wwc_core::group::game::{PlayedGroupGame, PreGroupGame};
use wwc_core::group::{Group, GroupError, GroupId, Groups};
use wwc_core::team::{Rank, Team, TeamId};
use wwc_core::Date;

pub fn try_groups_from_data(data: &Data) -> Result<Groups, LsvParseError> {
    let groups_with_err = data.groups.iter().map(|(id, group)| {
        let group: Result<Group, GroupError> = (group.clone()).try_into();
        (id, group)
    });
    if groups_with_err.clone().any(|(_, group)| group.is_err()) {
        Err(LsvParseError::GroupError)
    } else {
        Ok(groups_with_err
            .map(|(id, group)| (*id, group.unwrap()))
            .collect())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Data {
    pub teams: Vec<ParseTeam>,
    pub groups: HashMap<GroupId, ParseGroup>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct ParseTeam {
    id: TeamId,
    name: String,
    #[serde(rename = "fifaCode")]
    fifa_code: String,
    iso2: String,
    rank: Option<Rank>,
}

impl TryInto<Team> for ParseTeam {
    type Error = LsvParseError;
    fn try_into(self) -> Result<Team, Self::Error> {
        if let Some(rank) = self.rank {
            Ok(Team::new(
                self.id,
                self.name,
                self.fifa_code,
                self.iso2,
                rank,
            ))
        } else {
            Err(Self::Error::TeamError)
        }
    }
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
        Group::try_new(played_games, upcoming_games)
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

#[derive(Error, Debug)]
pub enum LsvParseError {
    #[error("Error parsing team")]
    TeamError,
    #[error("Error parsing group")]
    GroupError,
}
