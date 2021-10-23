//! LSV Fifa 2018 Playoff parsing
use crate::lsv::LsvParseError;
use serde::{Deserialize, Serialize};
use wwc_core::game::GameId;
use wwc_core::group::{GroupId, GroupOutcome};
use wwc_core::team::TeamId;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ParsePlayoff {
    pub round_16: ParseFirstRound,
}

impl ParsePlayoff {
    pub fn games<'a>(&'a self) -> impl Iterator<Item = &'a ParsePlayoffGame> {
        self.round_16.games.iter()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ParseFirstRound {
    #[serde(rename = "matches")]
    pub games: Vec<ParsePlayoffGame>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ParsePlayoffGame {
    #[serde(rename = "name")]
    pub id: GameId,
    pub home_team: ParsePlayoffTransition,
    pub away_team: ParsePlayoffTransition,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ParsePlayoffTransition {
    UnFinished(String),
    Finished(TeamId),
}

impl TryFrom<ParsePlayoffTransition> for GroupOutcome {
    type Error = LsvParseError;

    fn try_from(x: ParsePlayoffTransition) -> Result<Self, Self::Error> {
        match x {
            ParsePlayoffTransition::UnFinished(id) => ParsePlayoffTransition::parse_trans(&id),
            ParsePlayoffTransition::Finished(id) => Err(LsvParseError::TransitionComplete(id)),
        }
    }
}

impl ParsePlayoffTransition {
    pub fn team_from_finished(&self) -> Result<TeamId, LsvParseError> {
        match &self {
            Self::UnFinished(s) => Err(LsvParseError::TransitionIncomplete(String::from(s))),
            Self::Finished(id) => Ok(*id),
        }
    }
    fn parse_trans(trans: &str) -> Result<GroupOutcome, LsvParseError> {
        let mut s = trans.split('_');
        let outcome = s
            .next()
            .ok_or(LsvParseError::OutcomeParse(String::from(trans)))?;
        let id = s
            .next()
            .ok_or(LsvParseError::OutcomeParse(String::from(trans)))?;
        let id = id
            .chars()
            .next()
            .ok_or(LsvParseError::OutcomeParse(String::from(trans)))?;
        let id = GroupId::try_from(id.to_ascii_uppercase())?;
        match outcome {
            "winner" => Ok(GroupOutcome::Winner(id)),
            "runner" => Ok(GroupOutcome::RunnerUp(id)),
            _ => Err(LsvParseError::OutcomeParse(String::from(trans))),
        }
    }
}
