use serde::{Deserialize, Serialize};
use thiserror::Error;
use wwc_core::group::GroupError;
use wwc_core::group::Groups;
use wwc_core::playoff::transition::PlayoffTransitions;
use wwc_core::playoff::PlayoffError;
use wwc_core::team::{TeamId, Teams};

pub mod euro_2020;
pub mod fifa_2018;

pub use crate::lsv::euro_2020::Euro2020Data;
pub use crate::lsv::fifa_2018::Fifa2018Data;

pub fn get_data<T: LsvData>(data_path: &str) -> Result<T, LsvParseError> {
    let data = T::try_data_from_file(data_path)?;
    Ok(data)
}

pub trait LsvData: Sized {
    fn try_data_from_file(filename: &str) -> Result<Self, LsvParseError>;
    fn try_groups(&self) -> Result<Groups, LsvParseError>;
    fn try_teams(&self) -> Result<Teams, LsvParseError>;
    fn try_playoff_transitions(&self) -> Result<PlayoffTransitions, LsvParseError>;
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum GameType {
    Group,
    Qualified,
    Winner,
    Loser,
}

#[derive(Error, Debug)]
pub enum LsvParseError {
    #[error("File read error: {0}")]
    FileRead(#[from] std::io::Error),
    #[error("Deserialisation error: {0}")]
    Deserialisation(#[from] serde_json::Error),
    #[error("Error parsing team")]
    TeamParse,
    #[error("Error parsing group: {0}")]
    GroupParse(#[from] GroupError),
    #[error("Failed splitting '{0}' into 'outcome', 'id'")]
    OutcomeParse(String),
    #[error("Error parsing third place group id: {0}")]
    ThirdPlaceGroupId(String),
    #[error("Transition complete, got TeamId {0} instead of e.g. 'winner_b'")]
    TransitionComplete(TeamId),
    #[error("Error parsing playoff: {0}")]
    Playoff(#[from] PlayoffError),
}
