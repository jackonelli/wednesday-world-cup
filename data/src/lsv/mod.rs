use serde::{Deserialize, Serialize};
use thiserror::Error;
use wwc_core::group::GroupError;
use wwc_core::group::Groups;
use wwc_core::team::Teams;

pub mod euro_2020;
pub mod fifa_2018;

pub use euro_2020::Euro2020Data;
pub use fifa_2018::Fifa2018Data;

pub fn get_data<T: LsvData>(data_path: &str) -> Result<T, LsvParseError> {
    let data = T::try_data_from_file(data_path)?;
    Ok(data)
}

pub trait LsvData: Sized {
    fn try_data_from_file(filename: &str) -> Result<Self, LsvParseError>;
    fn try_groups(&self) -> Result<Groups, LsvParseError>;
    fn try_teams(&self) -> Result<Teams, LsvParseError>;
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
}
