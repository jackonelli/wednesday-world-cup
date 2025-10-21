//! LSV Euro 2020 JSON data interface
//!
//! Data source: <https://github.com/lsv/fifa-worldcup-2018>

pub mod group;
pub mod playoff;
use crate::file_io::read_json_file_to_str;
use crate::lsv::euro_2020::group::ParseGroup;
use crate::lsv::euro_2020::playoff::ParsePlayoff;
use crate::lsv::{LsvData, LsvParseError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use wwc_core::game::GameId;
use wwc_core::group::{GroupError, GroupId, GroupOutcome, Groups};
use wwc_core::playoff::TeamSource;
use wwc_core::playoff::transition::{PlayoffTransition, PlayoffTransitions};
use wwc_core::team::{FifaCode, Team, TeamId, TeamRank, Teams};

#[derive(Debug, Clone)]
pub struct Euro2020Data {
    teams: Vec<ParseTeam>,
    groups: Vec<ParseGroup>,
    pub team_map: TeamMap,
    playoff_trans: PlayoffTransitions,
    pub playoff: ParsePlayoff,
    pub team_sources: Vec<(GameId, (TeamSource, TeamSource))>,
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
            playoff: data.playoff.clone(),
            team_sources: Self::parse_team_sources(&data.playoff).collect(),
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
        // Only take the first 8 games, i.e. from the the first round.
        data.round16.games.iter().map(|game| {
            // TODO unwrap
            let home = GroupOutcome::try_from(game.qualification.home_team.clone()).unwrap();
            let away = GroupOutcome::try_from(game.qualification.away_team.clone()).unwrap();
            let trans = PlayoffTransition::new(home, away);
            (game.id, trans)
        })
    }

    pub(crate) fn parse_team_sources(
        data: &ParsePlayoff,
    ) -> impl Iterator<Item = (GameId, (TeamSource, TeamSource))> + '_ {
        data.games().map(|game| {
            let home = TeamSource::try_from(game.qualification.home_team.clone()).unwrap();
            let away = TeamSource::try_from(game.qualification.away_team.clone()).unwrap();
            (game.id, (home, away))
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
