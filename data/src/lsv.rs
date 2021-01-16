//! LSV JSON data interface
//!
//! The plan is to eventually move the data module to a separate crate.
//!
//! Data source: <https://github.com/lsv/fifa-worldcup-2018>
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use thiserror::Error;
use wwc_core::fair_play::{FairPlay, FairPlayScore};
use wwc_core::game::GoalCount;
use wwc_core::group::game::{PlayedGroupGame, Score, UnplayedGroupGame};
use wwc_core::group::{Group, GroupError, GroupId, Groups};
use wwc_core::team::{Rank, Team, TeamId};
use wwc_core::Date;

pub fn lsv_data_from_file(filename: &str) -> Data {
    let data_json =
        crate::file_io::read_json_file_to_str(filename).expect("Could not read from file");
    let data: Data = serde_json::from_str(&data_json).expect("JSON format error.");
    data
}

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Data {
    pub teams: Vec<ParseTeam>,
    pub groups: HashMap<GroupId, ParseGroup>,
}

impl Data {
    pub fn group_winners(&self) -> impl Iterator<Item = (&GroupId, &TeamId)> {
        self.groups.iter().map(|(id, group)| (id, &group.winner))
    }

    pub fn group_runner_ups(&self) -> impl Iterator<Item = (&GroupId, &TeamId)> {
        self.groups.iter().map(|(id, group)| (id, &group.runner_up))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ParseTeam {
    pub id: TeamId,
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
                &self.name,
                &self.fifa_code,
                &self.iso2,
                rank,
            ))
        } else {
            //Err(Self::Error::TeamError)
            //TODO: How to solve missing rank?
            Ok(Team::new(
                self.id,
                &self.name,
                &self.fifa_code,
                &self.iso2,
                Rank(0),
            ))
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParseGroup {
    name: String,
    winner: TeamId,
    #[serde(rename = "runnerup")]
    runner_up: TeamId,
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
            .collect::<Result<Vec<UnplayedGroupGame>, GroupError>>()?;

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
    home_result: Option<GoalCount>,
    away_result: Option<GoalCount>,
    home_penalty: Option<GoalCount>,
    away_penalty: Option<GoalCount>,
    home_fair_play: Option<FairPlay>,
    away_fair_play: Option<FairPlay>,
    finished: bool,
    date: Date,
}

impl TryInto<UnplayedGroupGame> for ParseGame {
    type Error = GroupError;
    fn try_into(self) -> Result<UnplayedGroupGame, Self::Error> {
        UnplayedGroupGame::try_new(self.id, self.home_team, self.away_team, self.date)
    }
}

impl TryInto<PlayedGroupGame> for ParseGame {
    type Error = GroupError;
    fn try_into(self) -> Result<PlayedGroupGame, Self::Error> {
        let game = UnplayedGroupGame::try_new(self.id, self.home_team, self.away_team, self.date)?;
        let score = match (self.home_result, self.away_result) {
            (Some(home), Some(away)) => Score::from((home, away)),
            _ => return Err(GroupError::GenericError),
        };
        let fair_play_score = match (self.home_fair_play, self.away_fair_play) {
            (Some(home), Some(away)) => FairPlayScore::new(home, away),
            _ => FairPlayScore::default(),
        };
        Ok(game.play(score, fair_play_score))
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
