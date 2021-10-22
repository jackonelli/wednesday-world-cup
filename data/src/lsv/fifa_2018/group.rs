//! LSV Fifa 2018 group stage parsing
use crate::lsv::GameType;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use wwc_core::fair_play::{FairPlay, FairPlayScore};
use wwc_core::game::{GameId, GoalCount};
use wwc_core::group::game::{GroupGameScore, PlayedGroupGame, UnplayedGroupGame};
use wwc_core::group::{Group, GroupError};
use wwc_core::team::TeamId;
use wwc_core::Date;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParseGroup {
    name: String,
    pub winner: Option<TeamId>,
    #[serde(rename = "runnerup")]
    pub runner_up: Option<TeamId>,
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
