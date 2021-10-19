//! LSV JSON data interface
//!
//! Data source: <https://github.com/lsv/fifa-worldcup-2018>

pub mod playoff;
use crate::file_io::read_json_file_to_str;
use crate::lsv::euro_2020::playoff::ParsePlayoff;
use crate::lsv::{GameType, LsvData, LsvParseError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use wwc_core::fair_play::{FairPlay, FairPlayScore};
use wwc_core::game::{GameId, GoalCount};
use wwc_core::group::game::{GroupGameScore, PlayedGroupGame, UnplayedGroupGame};
use wwc_core::group::{Group, GroupError, GroupId, GroupOutcome, Groups};
use wwc_core::playoff::transition::{PlayoffTransition, PlayoffTransitions};
use wwc_core::team::{FifaCode, Team, TeamId, TeamRank, Teams};
use wwc_core::Date;

#[derive(Debug, Clone)]
pub struct Euro2020Data {
    teams: Vec<ParseTeam>,
    groups: Vec<ParseGroup>,
    team_map: TeamMap,
    playoff_trans: PlayoffTransitions,
}

impl LsvData for Euro2020Data {
    fn try_data_from_file(filename: &str) -> Result<Euro2020Data, LsvParseError> {
        let data_json = read_json_file_to_str(filename)?;
        let data: ParseEuro2020Data = serde_json::from_str(&data_json)?;
        let groups = data
            .groups
            .into_iter()
            .map(|pg| {
                // Ugly, can be fixed with custom deserialisation, but I won't bother.
                let id = GroupId::try_from(char::from(pg.id).to_ascii_uppercase()).unwrap();
                ParseGroup { id, ..pg }
            })
            .collect::<Vec<ParseGroup>>();
        let team_map = Self::team_map(&data.teams);
        let trans = Self::parse_transitions(&data.playoff);
        let playoff_trans = PlayoffTransitions::try_new(
            trans,
            &groups.iter().map(|pg| pg.id).collect::<HashSet<GroupId>>(),
        )
        .map_err(LsvParseError::Playoff)?;
        Ok(Self {
            teams: data.teams,
            groups,
            team_map,
            playoff_trans,
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

    fn try_playoff_transitions(&self) -> Result<PlayoffTransitions, LsvParseError> {
        Ok(self.playoff_trans.clone())
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

    pub(crate) fn parse_transitions(
        data: &ParsePlayoff,
    ) -> impl Iterator<Item = (GameId, PlayoffTransition)> + '_ {
        data.games().map(|game| {
            // TODO unwrap
            let home = GroupOutcome::try_from(game.qualification.home_team.clone()).unwrap();
            let away = GroupOutcome::try_from(game.qualification.away_team.clone()).unwrap();
            let trans = PlayoffTransition::new(home, away);
            (game.id, trans)
        })
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

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ParseEuro2020Data {
    teams: Vec<ParseTeam>,
    groups: Vec<ParseGroup>,
    #[serde(rename = "knockoutphases")]
    playoff: ParsePlayoff,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct ParseTeam {
    #[serde(rename = "id")]
    fifa_code: String,
    name: String,
    rank: Option<TeamRank>,
}

type TeamMap = HashMap<String, TeamId>;

impl ParseTeam {
    fn try_parse_team(self, team_map: &TeamMap) -> Result<Team, LsvParseError> {
        let id = team_map.get(&self.fifa_code).unwrap();
        let team = if let Some(rank) = self.rank {
            Team::try_new(*id, &self.name, &self.fifa_code, rank)
        } else {
            //TODO: How to solve missing rank?
            Team::try_new(*id, &self.name, &self.fifa_code, TeamRank(0))
        };
        team.map_err(|_| LsvParseError::TeamParse)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ParseGroup {
    id: GroupId,
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
