//! LSV JSON data interface
//!
//! Data source: <https://github.com/lsv/fifa-worldcup-2018>
use crate::lsv::{GameType, LsvData, LsvParseError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wwc_core::fair_play::{FairPlay, FairPlayScore};
use wwc_core::game::{GameId, GoalCount};
use wwc_core::group::game::{GroupGameScore, PlayedGroupGame, UnplayedGroupGame};
use wwc_core::group::{Group, GroupError, GroupId, Groups};
use wwc_core::team::{FifaCode, Iso2, Team, TeamId, TeamRank, Teams};
use wwc_core::Date;

type TeamMap = HashMap<String, TeamId>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Euro2020Data {
    teams: Vec<ParseTeam>,
    groups: Vec<ParseGroup>,
    team_map: TeamMap,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ParseEuro2020Data {
    teams: Vec<ParseTeam>,
    groups: Vec<ParseGroup>,
}

impl LsvData for Euro2020Data {
    fn try_data_from_file(filename: &str) -> Result<Euro2020Data, LsvParseError> {
        let data_json = crate::file_io::read_json_file_to_str(filename)?;
        let mut data: ParseEuro2020Data = serde_json::from_str(&data_json)?;
        data.groups = data
            .groups
            .into_iter()
            .map(|mut pg| {
                // Ugly, can be fixed with custom deserialisation, but I won't bother.
                pg.id = GroupId::from(char::from(pg.id).to_ascii_uppercase());
                pg
            })
            .collect();
        let team_map = Self::team_map(&data.teams);
        Ok(Self {
            teams: data.teams,
            groups: data.groups,
            team_map,
        })
    }

    fn try_groups(&self) -> Result<Groups, LsvParseError> {
        Ok(self
            .groups
            .iter()
            .map(|pg| {
                let id = pg.id;
                pg.clone().try_parse_group(&self.team_map).map(|g| (id, g))
            })
            .collect::<Result<Groups, GroupError>>()?)
    }

    fn try_teams(&self) -> Result<Teams, LsvParseError> {
        let tmp = self
            .clone()
            .teams
            .into_iter()
            .map(|team| team.try_parse_team(&self.team_map))
            .collect::<Result<Vec<Team>, LsvParseError>>()?;
        Ok(tmp.into_iter().map(|t| (t.id, t)).collect())
    }
}

impl Euro2020Data {
    fn team_map(teams: &[ParseTeam]) -> TeamMap {
        teams
            .iter()
            .enumerate()
            .map(|(id, t)| (t.fifa_code.clone(), TeamId(id as u32)))
            .collect()
    }
}

/// Used for testing only
impl Euro2020Data {
    pub fn group_winners(&self) -> impl Iterator<Item = (GroupId, Option<FifaCode>)> + '_ {
        self.groups.iter().map(|pg| (pg.id, pg.winner.clone()))
    }

    pub fn group_runner_ups(&self) -> impl Iterator<Item = (GroupId, Option<FifaCode>)> + '_ {
        self.groups.iter().map(|pg| (pg.id, pg.runner_up.clone()))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct ParseTeam {
    #[serde(rename = "id")]
    fifa_code: String,
    name: String,
    rank: Option<TeamRank>,
}

impl ParseTeam {
    fn try_parse_team(self, team_map: &TeamMap) -> Result<Team, LsvParseError> {
        let id = team_map.get(&self.fifa_code).unwrap();
        if let Some(rank) = self.rank {
            Ok(Team::new(
                *id,
                &self.name,
                &self.fifa_code,
                Iso2::from(&FifaCode::from(self.fifa_code.clone())).as_ref(),
                rank,
            ))
        } else {
            //Err(Self::Error::TeamError)
            //TODO: How to solve missing rank?
            Ok(Team::new(
                *id,
                &self.name,
                &self.fifa_code,
                &String::from(Iso2::from(&FifaCode::from(self.fifa_code.clone()))),
                TeamRank(0),
            ))
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ParseGroup {
    id: GroupId,
    name: String,
    winner: Option<FifaCode>,
    #[serde(rename = "runnerup")]
    runner_up: Option<FifaCode>,
    #[serde(rename = "matches")]
    games: Vec<ParseGame>,
}

impl ParseGroup {
    fn try_parse_group(self, team_map: &TeamMap) -> Result<Group, GroupError> {
        let upcoming_games = self
            .games
            .iter()
            .filter(|game| !game.finished)
            .map(|game| ParseGame::try_parse_unplayed(game.clone(), team_map))
            .collect::<Result<Vec<UnplayedGroupGame>, GroupError>>()?;

        let played_games = self
            .games
            .iter()
            .filter(|game| game.finished)
            .map(|game| ParseGame::try_parse_played(game.clone(), team_map))
            .collect::<Result<Vec<PlayedGroupGame>, GroupError>>()?;
        Group::try_new(upcoming_games, played_games)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ParseGame {
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

impl ParseGame {
    fn try_parse_unplayed(
        parse_game: ParseGame,
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
        parse_game: ParseGame,
        team_map: &TeamMap,
    ) -> Result<PlayedGroupGame, GroupError> {
        println!("{:?}", parse_game.home_team);
        let game = UnplayedGroupGame::try_new(
            GameId::from(parse_game.id),
            *team_map.get(&parse_game.home_team).unwrap(),
            *team_map.get(&parse_game.away_team).unwrap(),
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
