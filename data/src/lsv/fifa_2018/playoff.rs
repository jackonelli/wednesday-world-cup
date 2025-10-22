//! LSV Fifa 2018 Playoff parsing
use crate::lsv::LsvParseError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wwc_core::game::{GameId, GoalCount};
use wwc_core::group::{GroupId, GroupOutcome};
use wwc_core::playoff::{PlayoffGameState, PlayoffResult, PlayoffScore, TeamSource};
use wwc_core::team::TeamId;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ParsePlayoff {
    pub round_16: ParseRound,
    pub round_8: ParseRound,
    pub round_4: ParseRound,
    pub round_2: ParseRound,
    pub round_2_loser: ParseRound,
}

impl ParsePlayoff {
    pub fn games(&self) -> impl Iterator<Item = &ParsePlayoffGame> {
        self.round_16
            .games
            .iter()
            .chain(self.round_8.games.iter())
            .chain(self.round_4.games.iter())
            .chain(self.round_2_loser.games.iter())
            .chain(self.round_2.games.iter())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ParseRound {
    #[serde(rename = "matches")]
    pub games: Vec<ParsePlayoffGame>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ParsePlayoffGame {
    #[serde(rename = "name")]
    pub id: GameId,
    #[serde(rename = "type")]
    pub game_type: String,
    pub home_team: ParsePlayoffTransition,
    pub away_team: ParsePlayoffTransition,
    pub home_result: Option<u32>,
    pub away_result: Option<u32>,
    pub home_penalty: Option<u32>,
    pub away_penalty: Option<u32>,
    pub winner: Option<TeamId>,
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
    /// Convert to TeamSource given the game type context
    pub fn to_team_source(&self, game_type: &str) -> Result<TeamSource, LsvParseError> {
        match self {
            ParsePlayoffTransition::UnFinished(trans) => Ok(TeamSource::GroupOutcome(
                ParsePlayoffTransition::parse_trans(trans)?,
            )),
            ParsePlayoffTransition::Finished(id) => {
                // In subsequent rounds, the ID is actually a GameId, not a TeamId
                let game_id = GameId::from(id.0);
                match game_type {
                    "winner" | "qualified" => Ok(TeamSource::WinnerOf(game_id)),
                    "loser" => Ok(TeamSource::LoserOf(game_id)),
                    _ => Err(LsvParseError::OutcomeParse(format!(
                        "Unknown game type: {}",
                        game_type
                    ))),
                }
            }
        }
    }
}

impl TryFrom<ParsePlayoffTransition> for TeamSource {
    type Error = LsvParseError;

    fn try_from(value: ParsePlayoffTransition) -> Result<Self, Self::Error> {
        match value {
            ParsePlayoffTransition::UnFinished(trans) => Ok(TeamSource::GroupOutcome(
                ParsePlayoffTransition::parse_trans(&trans)?,
            )),
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
            .ok_or_else(|| LsvParseError::OutcomeParse(String::from(trans)))?;
        let id = s
            .next()
            .ok_or_else(|| LsvParseError::OutcomeParse(String::from(trans)))?;
        let id = id
            .chars()
            .next()
            .ok_or_else(|| LsvParseError::OutcomeParse(String::from(trans)))?;
        let id = GroupId::try_from(id.to_ascii_uppercase())?;
        match outcome {
            "winner" => Ok(GroupOutcome::Winner(id)),
            "runner" => Ok(GroupOutcome::RunnerUp(id)),
            _ => Err(LsvParseError::OutcomeParse(String::from(trans))),
        }
    }
}

impl ParsePlayoffGame {
    pub fn try_parse(
        self,
        teams_map: &HashMap<TeamId, TeamId>,
    ) -> Result<PlayoffGameState, LsvParseError> {
        let home_team_id = match &self.home_team {
            ParsePlayoffTransition::Finished(id) => Some(*id),
            ParsePlayoffTransition::UnFinished(_) => None,
        };
        let away_team_id = match &self.away_team {
            ParsePlayoffTransition::Finished(id) => Some(*id),
            ParsePlayoffTransition::UnFinished(_) => None,
        };

        match (home_team_id, away_team_id) {
            (None, None) => Ok(PlayoffGameState::Pending {
                game_id: self.id,
                home_source: TeamSource::try_from(self.home_team)?,
                away_source: TeamSource::try_from(self.away_team)?,
            }),
            (Some(home), None) => Ok(PlayoffGameState::HomeKnown {
                game_id: self.id,
                home: *teams_map
                    .get(&home)
                    .ok_or(LsvParseError::MissingTeamId(home))?,
                away_source: TeamSource::try_from(self.away_team)?,
            }),
            (None, Some(away)) => Ok(PlayoffGameState::AwayKnown {
                game_id: self.id,
                home_source: TeamSource::try_from(self.home_team)?,
                away: *teams_map
                    .get(&away)
                    .ok_or(LsvParseError::MissingTeamId(away))?,
            }),
            (Some(home), Some(away)) => match (self.home_result, self.away_result) {
                (None, None) => Ok(PlayoffGameState::Ready {
                    game_id: self.id,
                    home: *teams_map
                        .get(&home)
                        .ok_or(LsvParseError::MissingTeamId(home))?,
                    away: *teams_map
                        .get(&away)
                        .ok_or(LsvParseError::MissingTeamId(away))?,
                }),
                (Some(home_result), Some(away_result)) => Ok(PlayoffGameState::Played {
                    game_id: self.id,
                    result: PlayoffResult::new(
                        *teams_map
                            .get(&home)
                            .ok_or(LsvParseError::MissingTeamId(home))?,
                        *teams_map
                            .get(&away)
                            .ok_or(LsvParseError::MissingTeamId(away))?,
                        PlayoffScore::try_new(
                            GoalCount::try_from(home_result)?,
                            GoalCount::try_from(away_result)?,
                            self.home_penalty
                                .map(|p| GoalCount::try_from(p))
                                .transpose()?,
                            self.away_penalty
                                .map(|p| GoalCount::try_from(p))
                                .transpose()?,
                        )?,
                    ),
                }),
                _ => Err(LsvParseError::MissingResult),
            },
        }
    }
}
