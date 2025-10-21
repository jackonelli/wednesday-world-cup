use crate::lsv::LsvParseError;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use wwc_core::game::{GameId, GoalCount};
use wwc_core::group::{GroupError, GroupId, GroupOutcome};
use wwc_core::playoff::{PlayoffGameState, PlayoffResult, PlayoffScore, TeamSource};
use wwc_core::team::{FifaCode, TeamId};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParsePlayoff {
    pub round16: ParseRound,
    pub round8: ParseRound,
    pub round4: ParseRound,
    pub round2: ParseRound,
}

impl ParsePlayoff {
    pub fn games(&self) -> impl Iterator<Item = &ParsePlayoffGame> {
        self.round16
            .games
            .iter()
            .chain(self.round8.games.iter())
            .chain(self.round4.games.iter())
            .chain(self.round2.games.iter())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParseRound {
    #[serde(rename = "matches")]
    pub games: Vec<ParsePlayoffGame>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParsePlayoffGame {
    pub id: GameId,
    pub qualification: ParseQualification,
    // Fifa code
    pub home_team: Option<FifaCode>,
    // Fifa code
    pub away_team: Option<FifaCode>,
    home_result: Option<GoalCount>,
    away_result: Option<GoalCount>,
    home_penalty: Option<GoalCount>,
    away_penalty: Option<GoalCount>,
    pub winner: Option<FifaCode>,
}

impl ParsePlayoffGame {
    pub fn try_parse(
        self,
        teams_map: &HashMap<FifaCode, TeamId>,
    ) -> Result<PlayoffGameState, LsvParseError> {
        match (self.home_team, self.away_team) {
            (None, None) => Ok(PlayoffGameState::Pending {
                game_id: self.id,
                home_source: TeamSource::try_from(self.qualification.home_team)?,
                away_source: TeamSource::try_from(self.qualification.away_team)?,
            }),
            (Some(home_team), None) => Ok(PlayoffGameState::HomeKnown {
                game_id: self.id,
                home: teams_map
                    .get(&home_team)
                    .ok_or(LsvParseError::MissingTeam(home_team))?
                    .clone(),
                away_source: TeamSource::try_from(self.qualification.away_team)?,
            }),
            (None, Some(away_team)) => Ok(PlayoffGameState::AwayKnown {
                game_id: self.id,
                home_source: TeamSource::try_from(self.qualification.home_team)?,
                away: teams_map
                    .get(&away_team)
                    .ok_or(LsvParseError::MissingTeam(away_team))?
                    .clone(),
            }),
            (Some(home_team), Some(away_team)) => match (self.home_result, self.away_result) {
                (None, None) => Ok(PlayoffGameState::Ready {
                    game_id: self.id,
                    home: teams_map
                        .get(&home_team)
                        .ok_or(LsvParseError::MissingTeam(home_team))?
                        .clone(),
                    away: teams_map
                        .get(&away_team)
                        .ok_or(LsvParseError::MissingTeam(away_team))?
                        .clone(),
                }),
                (Some(home_result), Some(away_result)) => Ok(PlayoffGameState::Played {
                    game_id: self.id,
                    result: PlayoffResult::new(
                        teams_map
                            .get(&home_team)
                            .ok_or(LsvParseError::MissingTeam(home_team))?
                            .clone(),
                        teams_map
                            .get(&away_team)
                            .ok_or(LsvParseError::MissingTeam(away_team))?
                            .clone(),
                        PlayoffScore::try_new(
                            home_result,
                            away_result,
                            self.home_penalty,
                            self.away_penalty,
                        )?,
                    ),
                }),
                _ => Err(LsvParseError::MissingResult),
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParseQualification {
    pub home_team: ParseQualificationTeam,
    pub away_team: ParseQualificationTeam,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParseQualificationTeam {
    qualificationtype: ParseGroupOutcome,
    group: Option<String>,
    #[serde(rename = "match")]
    game: Option<GameId>,
}

impl TryFrom<ParseQualificationTeam> for TeamSource {
    type Error = LsvParseError;

    fn try_from(value: ParseQualificationTeam) -> Result<Self, Self::Error> {
        match (&value.group, &value.game) {
            (Some(_), None) => Ok(TeamSource::GroupOutcome(GroupOutcome::try_from(value)?)),
            (None, Some(game_id)) => match &value.qualificationtype {
                ParseGroupOutcome::Winner => Ok(TeamSource::WinnerOf(*game_id)),
                ParseGroupOutcome::Loser => Ok(TeamSource::LoserOf(*game_id)),
                _ => Err(LsvParseError::GroupOutcomeInPlayoffQualification),
            },
            _ => Err(LsvParseError::InvalidQualification),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParseGroupOutcome {
    Winner,
    Loser,
    RunnerUp,
    ThirdPlace,
}

impl TryFrom<ParseQualificationTeam> for GroupOutcome {
    type Error = LsvParseError;

    fn try_from(x: ParseQualificationTeam) -> Result<Self, Self::Error> {
        if let None = x.group {
            return Err(LsvParseError::OutcomeParse(
                "No group tag found in round16 game".into(),
            ));
        }
        match x.qualificationtype {
            ParseGroupOutcome::Winner => {
                let id = parse_single_group_id(&x.group.unwrap())?;
                Ok(GroupOutcome::Winner(id))
            }
            ParseGroupOutcome::RunnerUp => {
                let id = parse_single_group_id(&x.group.unwrap())?;
                Ok(GroupOutcome::RunnerUp(id))
            }
            ParseGroupOutcome::ThirdPlace => {
                let ids = parse_group_id_set(&x.group.unwrap())?;
                Ok(GroupOutcome::ThirdPlace(ids))
            }
            _ => Err(LsvParseError::PlayoffOutcomeInQualification),
        }
    }
}

fn parse_single_group_id(id: &str) -> Result<GroupId, LsvParseError> {
    if id.len() == 1 {
        let idc = id.chars().next().unwrap();
        let idc = GroupId::try_from(idc.to_ascii_uppercase())
            .map_err(|_err| LsvParseError::ThirdPlaceGroupId(String::from(id)))?;
        Ok(idc)
    } else {
        Err(LsvParseError::ThirdPlaceGroupId(String::from(id)))
    }
}

fn parse_group_id_set(ids: &str) -> Result<HashSet<GroupId>, LsvParseError> {
    let chars = ids.split('/');
    let (mut count, chars) = chars.tee();
    if count.all(|x| x.len() == 1) {
        let id_set = chars
            .map(|id| id.chars().next().unwrap())
            .map(|c| GroupId::try_from(c.to_ascii_uppercase()))
            .collect::<Result<HashSet<GroupId>, GroupError>>()
            .map_err(LsvParseError::GroupParse)?;
        match id_set.len() {
            3..=4 => Ok(id_set),
            _ => Err(LsvParseError::ThirdPlaceGroupId(String::from(ids))),
        }
    } else {
        Err(LsvParseError::ThirdPlaceGroupId(String::from(ids)))
    }
}

#[cfg(test)]
mod lsv_euro_2020_tests {
    use super::*;
    use std::iter::FromIterator;

    #[test]
    fn group_id_set_parsing_three_groups() {
        let raw = String::from("D/E/F");
        let true_set: HashSet<GroupId> = HashSet::from_iter(
            vec!['D', 'E', 'F']
                .iter()
                .map(|id| GroupId::try_from(*id).unwrap()),
        );
        let parsed = parse_group_id_set(&raw).unwrap();
        println!("{:?}", parsed);
        assert_eq!(true_set, parsed);
    }

    #[test]
    fn group_id_set_parsing_four_groups() {
        let raw = String::from("A/B/C/D");
        let true_set: HashSet<GroupId> = HashSet::from_iter(
            vec!['A', 'B', 'C', 'D']
                .iter()
                .map(|id| GroupId::try_from(*id).unwrap()),
        );
        let parsed = parse_group_id_set(&raw).unwrap();
        println!("{:?}", parsed);
        assert_eq!(true_set, parsed);
    }
}
