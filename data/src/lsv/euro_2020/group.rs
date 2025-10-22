//! LSV Euro 2020 Playoff parse
//!
//! Data source: <https://github.com/lsv/fifa-worldcup-2018>

use crate::lsv::GameType;
use crate::lsv::euro_2020::TeamMap;
use serde::{Deserialize, Serialize};
use wwc_core::Date;
use wwc_core::fair_play::{FairPlay, FairPlayScore};
use wwc_core::game::{GameId, GoalCount};
use wwc_core::group::game::{GroupGameScore, PlayedGroupGame, UnplayedGroupGame};
use wwc_core::group::{Group, GroupError, GroupId};
use wwc_core::team::FifaCode;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct ParseGroup {
    pub(crate) id: GroupId,
    pub(crate) winner: Option<FifaCode>,
    #[serde(rename = "runnerup")]
    pub(crate) runner_up: Option<FifaCode>,
    #[serde(rename = "matches")]
    pub(crate) games: Vec<ParseGroupGame>,
}

impl ParseGroup {
    pub(crate) fn try_parse_group(self, team_map: &TeamMap) -> Result<Group, GroupError> {
        let upcoming_games = self
            .games
            .iter()
            .filter(|game| !game.finished)
            .map(|game| ParseGroupGame::try_parse_unplayed(game.clone(), team_map))
            .collect::<Result<Vec<UnplayedGroupGame>, GroupError>>()?;

        let played_games = self
            .games
            .iter()
            .filter(|game| game.finished)
            .map(|game| ParseGroupGame::try_parse_played(game.clone(), team_map))
            .collect::<Result<Vec<PlayedGroupGame>, GroupError>>()?;
        Group::try_new(upcoming_games, played_games)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct ParseGroupGame {
    id: u32,
    #[serde(rename = "matchtype")]
    type_: GameType,
    home_team: String,
    away_team: String,
    home_result: Option<GoalCount>,
    away_result: Option<GoalCount>,
    home_penalty: Option<GoalCount>,
    away_penalty: Option<GoalCount>,
    home_fair_play: Option<FairPlay>,
    away_fair_play: Option<FairPlay>,
    finished: bool,
    date: Date,
}

impl ParseGroupGame {
    fn try_parse_unplayed(
        parse_game: ParseGroupGame,
        team_map: &TeamMap,
    ) -> Result<UnplayedGroupGame, GroupError> {
        UnplayedGroupGame::try_new(
            GameId::from(parse_game.id),
            *team_map.get(&parse_game.home_team).unwrap(),
            *team_map.get(&parse_game.away_team).unwrap(),
            parse_game.date,
        )
    }

    fn try_parse_played(
        parse_game: ParseGroupGame,
        team_map: &TeamMap,
    ) -> Result<PlayedGroupGame, GroupError> {
        let game = UnplayedGroupGame::try_new(
            GameId::from(parse_game.id),
            *team_map.get(&parse_game.home_team).unwrap(),
            *team_map.get(&parse_game.away_team).unwrap(),
            parse_game.date,
        )?;
        let score = match (parse_game.home_result, parse_game.away_result) {
            (Some(home), Some(away)) => GroupGameScore::new(home, away),
            _ => return Err(GroupError::GenericError),
        };
        let fair_play_score = match (parse_game.home_fair_play, parse_game.away_fair_play) {
            (Some(home), Some(away)) => FairPlayScore::new(home, away),
            _ => FairPlayScore::default(),
        };
        Ok(game.play(score, fair_play_score))
    }
}
