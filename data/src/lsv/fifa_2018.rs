//! LSV JSON data interface
//!
//! Data source: <https://github.com/lsv/fifa-worldcup-2018>
use crate::file_io::read_json_file_to_str;
use crate::lsv::{GameType, LsvData, LsvParseError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use wwc_core::fair_play::{FairPlay, FairPlayScore};
use wwc_core::game::GoalCount;
use wwc_core::group::game::{GroupGameScore, PlayedGroupGame, UnplayedGroupGame};
use wwc_core::group::{Group, GroupError, GroupId, Groups};
use wwc_core::playoff::transition::PlayoffTransitions;
use wwc_core::team::{Team, TeamId, TeamRank, Teams};
use wwc_core::Date;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Fifa2018Data {
    teams: Vec<ParseTeam>,
    groups: HashMap<GroupId, ParseGroup>,
}

impl LsvData for Fifa2018Data {
    fn try_data_from_file(filename: &str) -> Result<Fifa2018Data, LsvParseError> {
        let data_json = read_json_file_to_str(filename)?;
        let mut data: Fifa2018Data = serde_json::from_str(&data_json)?;
        data.groups = data
            .groups
            .into_iter()
            .map(|(id, pg)| (id.into_uppercase(), pg))
            .collect();
        Ok(data)
    }

    fn try_groups(&self) -> Result<Groups, LsvParseError> {
        Ok(self
            .groups
            .iter()
            .map(|(id, group)| {
                group.clone().try_into().map(|g| {
                    (
                        GroupId::try_from(char::from(*id).to_ascii_uppercase()).unwrap(),
                        g,
                    )
                })
            })
            .collect::<Result<Groups, GroupError>>()?)
    }

    fn try_teams(&self) -> Result<Teams, LsvParseError> {
        let tmp = self
            .clone()
            .teams
            .into_iter()
            .map(|team| team.try_into())
            .collect::<Result<Vec<Team>, LsvParseError>>()?;
        Ok(tmp.into_iter().map(|t| (t.id, t)).collect())
    }
    fn try_playoff_transitions(&self) -> Result<PlayoffTransitions, LsvParseError> {
        todo!()
    }
}

/// Used for testing only
impl Fifa2018Data {
    pub fn group_winners(&self) -> impl Iterator<Item = (GroupId, Option<TeamId>)> + '_ {
        self.groups.iter().map(|(id, pg)| (*id, pg.winner))
    }

    pub fn group_runner_ups(&self) -> impl Iterator<Item = (GroupId, Option<TeamId>)> + '_ {
        self.groups.iter().map(|(id, pg)| (*id, pg.runner_up))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct ParseTeam {
    id: TeamId,
    name: String,
    #[serde(rename = "fifaCode")]
    fifa_code: String,
    iso2: String,
    rank: Option<TeamRank>,
}

impl TryFrom<ParseTeam> for Team {
    type Error = LsvParseError;
    fn try_from(parse_team: ParseTeam) -> Result<Team, Self::Error> {
        let team = if let Some(rank) = parse_team.rank {
            Team::try_new(parse_team.id, &parse_team.name, &parse_team.fifa_code, rank)
        } else {
            //Err(Self::Error::TeamError)
            //TODO: How to solve missing rank?
            Team::try_new(
                parse_team.id,
                &parse_team.name,
                &parse_team.fifa_code,
                TeamRank(0),
            )
        };
        team.map_err(|_| LsvParseError::TeamParse)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ParseGroup {
    name: String,
    winner: Option<TeamId>,
    #[serde(rename = "runnerup")]
    runner_up: Option<TeamId>,
    #[serde(rename = "matches")]
    games: Vec<ParseGame>,
}

impl TryFrom<ParseGroup> for Group {
    type Error = GroupError;
    fn try_from(parse_group: ParseGroup) -> Result<Group, Self::Error> {
        let upcoming_games = parse_group
            .games
            .iter()
            .filter(|game| !game.finished)
            .map(|game| {
                let game = *game;
                game.try_into()
            })
            .collect::<Result<Vec<UnplayedGroupGame>, GroupError>>()?;

        let played_games = parse_group
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
struct ParseGame {
    #[serde(rename = "name")]
    id: u32,
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

impl TryFrom<ParseGame> for UnplayedGroupGame {
    type Error = GroupError;
    fn try_from(parse_game: ParseGame) -> Result<UnplayedGroupGame, Self::Error> {
        UnplayedGroupGame::try_new(
            parse_game.id,
            parse_game.home_team,
            parse_game.away_team,
            parse_game.date,
        )
    }
}

impl TryFrom<ParseGame> for PlayedGroupGame {
    type Error = GroupError;
    fn try_from(parse_game: ParseGame) -> Result<PlayedGroupGame, Self::Error> {
        let game = UnplayedGroupGame::try_new(
            parse_game.id,
            parse_game.home_team,
            parse_game.away_team,
            parse_game.date,
        )?;
        let score = match (parse_game.home_result, parse_game.away_result) {
            (Some(home), Some(away)) => GroupGameScore::from((home, away)),
            _ => return Err(GroupError::GenericError),
        };
        let fair_play_score = match (parse_game.home_fair_play, parse_game.away_fair_play) {
            (Some(home), Some(away)) => FairPlayScore::new(home, away),
            _ => FairPlayScore::default(),
        };
        Ok(game.play(score, fair_play_score))
    }
}
