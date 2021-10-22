//! LSV JSON data interface
//!
//! Data source: <https://github.com/lsv/fifa-worldcup-2018>
mod group;
mod playoff;
use crate::file_io::read_json_file_to_str;
use crate::lsv::fifa_2018::group::ParseGroup;
use crate::lsv::fifa_2018::playoff::ParsePlayoff;
use crate::lsv::{LsvData, LsvParseError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::convert::{TryFrom, TryInto};
use wwc_core::game::GameId;
use wwc_core::group::{GroupError, GroupId, GroupOutcome, Groups};
use wwc_core::playoff::transition::{PlayoffTransition, PlayoffTransitions};
use wwc_core::team::{Team, TeamId, TeamRank, Teams};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Fifa2018Data {
    teams: Vec<ParseTeam>,
    groups: HashMap<GroupId, ParseGroup>,
    #[serde(rename = "knockout")]
    playoff: ParsePlayoff,
}

impl LsvData for Fifa2018Data {
    fn try_playoff_transitions(&self) -> Result<PlayoffTransitions, LsvParseError> {
        let trans = self
            .playoff
            .round_16
            .games
            .iter()
            .map(|game| {
                let home = GroupOutcome::try_from(game.home_team.clone());
                let away = GroupOutcome::try_from(game.away_team.clone());
                match (home, away) {
                    (Ok(home), Ok(away)) => Ok((game.id, PlayoffTransition::new(home, away))),
                    _ => Err(LsvParseError::OutcomeParse(format!(
                        "home: {:?}, away: {:?}",
                        game.home_team, game.away_team
                    ))),
                }
            })
            .collect::<Result<BTreeMap<GameId, PlayoffTransition>, LsvParseError>>()?;
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
