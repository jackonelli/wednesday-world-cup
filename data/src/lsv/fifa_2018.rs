//! LSV JSON data interface
//!
//! Data source: <https://github.com/lsv/fifa-worldcup-2018>
use crate::file_io::read_json_file_to_str;
use crate::lsv::{GameType, LsvData, LsvParseError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::convert::{TryFrom, TryInto};
use wwc_core::fair_play::{FairPlay, FairPlayScore};
use wwc_core::game::{GameId, GoalCount};
use wwc_core::group::game::{GroupGameScore, PlayedGroupGame, UnplayedGroupGame};
use wwc_core::group::{Group, GroupError, GroupId, Groups, GroupOutcome};
use wwc_core::team::{Team, TeamId, TeamRank, Teams};
use wwc_core::Date;
use wwc_core::playoff::transition::{PlayoffTransition, PlayoffTransitions};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Fifa2018Data {
    teams: Vec<ParseTeam>,
    groups: HashMap<GroupId, ParseGroup>,
    #[serde(rename = "knockout")]
    playoff: ParsePlayoff,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct ParsePlayoff {
    round_16: ParseFirstRound,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct ParseFirstRound {
    #[serde(rename = "matches")]
    games: Vec<ParsePlayoffGame>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct ParsePlayoffGame {
    #[serde(rename = "name")]
    id: GameId,
    home_team: ParsePlayoffTransition,
    away_team: ParsePlayoffTransition,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
enum ParsePlayoffTransition {
    UnFinished(String),
    Finished(TeamId),
}

impl TryFrom<ParsePlayoffTransition> for GroupOutcome {
    type Error = LsvParseError;

    fn try_from(x: ParsePlayoffTransition) -> Result<Self, Self::Error> {
        match x {
        ParsePlayoffTransition::UnFinished(id) => ParsePlayoffTransition::parse_trans(&id),
        ParsePlayoffTransition::Finished(id) => Err(LsvParseError::TransitionComplete(id))
        }
    }
}

impl ParsePlayoffTransition {
    fn parse_trans(trans: &str) -> Result<GroupOutcome, LsvParseError> {
        let mut s = trans.split('_');
        let outcome = s.next().ok_or(LsvParseError::OutcomeParse(String::from(trans)))?;
        let id = s.next().ok_or(LsvParseError::OutcomeParse(String::from(trans)))?;
        let id = id.chars().next().ok_or(LsvParseError::OutcomeParse(String::from(trans)))?;
        let id = GroupId::try_from(id.to_ascii_uppercase())?;
        match outcome {
            "winner" => Ok(GroupOutcome::Winner(id)),
            "runner" => Ok(GroupOutcome::RunnerUp(id)),
            _ => Err(LsvParseError::OutcomeParse(String::from(trans)))
        }
    }
}

impl LsvData for Fifa2018Data {
    fn try_playoff_transitions(&self) -> Result<PlayoffTransitions, LsvParseError> {
        let trans = self.playoff.round_16.games.iter().map(|game| {
            let home = GroupOutcome::try_from(game.home_team.clone());
            let away = GroupOutcome::try_from(game.away_team.clone());
            match (home, away) {
                (Ok(home), Ok(away)) => Ok((game.id, PlayoffTransition::new(home, away))),
                _ => Err(LsvParseError::OutcomeParse(format!("home: {:?}, away: {:?}", game.home_team, game.away_team)))
            }
        }).collect::<Result<BTreeMap<GameId, PlayoffTransition>, LsvParseError>>()?;
        let groups = self.try_groups()?.iter().map(|(id, _)| *id).collect();
        Ok(PlayoffTransitions::try_new(trans.into_iter(), &groups)?)
    }

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
    games: Vec<ParseGroupGame>,
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
struct ParseGroupGame {
    #[serde(rename = "name")]
    id: GameId,
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

impl TryFrom<ParseGroupGame> for UnplayedGroupGame {
    type Error = GroupError;
    fn try_from(parse_game: ParseGroupGame) -> Result<UnplayedGroupGame, Self::Error> {
        UnplayedGroupGame::try_new(
            parse_game.id,
            parse_game.home_team,
            parse_game.away_team,
            parse_game.date,
        )
    }
}

impl TryFrom<ParseGroupGame> for PlayedGroupGame {
    type Error = GroupError;
    fn try_from(parse_game: ParseGroupGame) -> Result<PlayedGroupGame, Self::Error> {
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
