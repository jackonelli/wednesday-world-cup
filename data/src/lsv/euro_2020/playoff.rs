use crate::lsv::LsvParseError;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::convert::TryFrom;
use wwc_core::game::GameId;
use wwc_core::group::{GroupError, GroupId, GroupOutcome};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParsePlayoff {
    round16: ParseFirstRound,
}

impl ParsePlayoff {
    pub fn games(&self) -> impl Iterator<Item = &ParsePlayoffGame> {
        self.round16.games.iter()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ParseFirstRound {
    #[serde(rename = "matches")]
    games: Vec<ParsePlayoffGame>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParsePlayoffGame {
    pub id: GameId,
    pub(crate) qualification: ParseQualification,
    // Fifa code
    pub home_team: Option<String>,
    // Fifa code
    pub away_team: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct ParseQualification {
    pub(crate) home_team: ParseQualificationTeam,
    pub(crate) away_team: ParseQualificationTeam,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct ParseQualificationTeam {
    qualificationtype: ParseGroupOutcome,
    group: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParseGroupOutcome {
    Winner,
    RunnerUp,
    ThirdPlace,
}

impl TryFrom<ParseQualificationTeam> for GroupOutcome {
    type Error = LsvParseError;

    fn try_from(x: ParseQualificationTeam) -> Result<Self, Self::Error> {
        match x.qualificationtype {
            ParseGroupOutcome::Winner => {
                let id = parse_single_group_id(&x.group)?;
                Ok(GroupOutcome::Winner(id))
            }
            ParseGroupOutcome::RunnerUp => {
                let id = parse_single_group_id(&x.group)?;
                Ok(GroupOutcome::RunnerUp(id))
            }
            ParseGroupOutcome::ThirdPlace => {
                let ids = parse_group_id_set(&x.group)?;
                Ok(GroupOutcome::ThirdPlace(ids))
            }
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
