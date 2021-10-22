//! Tournament participants (e.g. countries)
use derive_more::{AsRef, Display, From, Into};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

/// Internal numerical Team id
#[derive(
    Deserialize,
    Serialize,
    Debug,
    Clone,
    Copy,
    Display,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::hash::Hash,
    From,
    Into,
)]
pub struct TeamId(pub u32);

/// Tournament participant
///
/// This struct is rarely used internally in the core crate.
/// There, teams are instead identified with the leaner [`TeamId`].
///
/// This struct is used in parsing and for generating visual representation.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Team {
    pub id: TeamId,
    pub name: TeamName,
    #[serde(rename = "fifaCode")]
    pub fifa_code: FifaCode,
    pub rank: TeamRank,
}

impl Team {
    /// Fallible constructor
    ///
    /// It's fallible because the function accepts unchecked string slices for convenience.
    /// The conversion into proper types might fail.
    pub fn try_new(
        id: TeamId,
        name: &str,
        fifa_code: &str,
        rank: TeamRank,
    ) -> Result<Self, TeamError> {
        FifaCode::try_from(String::from(fifa_code)).map(|fifa_code| Self {
            id,
            name: TeamName(String::from(name)),
            fifa_code: FifaCode(String::from(fifa_code)),
            rank,
        })
    }

    pub fn iso2(&self) -> Iso2 {
        Iso2::from(&self.fifa_code)
    }
}

impl std::fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Display name
///
/// E.g. "Sweden"
#[derive(Display, Debug, Clone, AsRef, From, Into, Deserialize, Serialize, PartialEq)]
#[as_ref(forward)]
pub struct TeamName(pub(crate) String);

/// ISO2 codes
#[derive(Display, Debug, Clone, AsRef, From, Into, Deserialize, Serialize, PartialEq)]
#[as_ref(forward)]
pub struct Iso2(String);

/// Fifa's country identifier
///
/// A Fifa code is a trigram or trigraph, i.e. a three letter id.
/// Furthermore, the codes are all upper case ASCII letters, e.g. "DEN"
/// <https://en.wikipedia.org/wiki/List_of_FIFA_country_codes>
#[derive(Display, Debug, Clone, Into, AsRef, Deserialize, Serialize, PartialEq)]
#[as_ref(forward)]
pub struct FifaCode(String);

/// Fallible conversion String -> FifaCode
///
/// Enforces the rules specified in [`FifaCode`]
impl TryFrom<String> for FifaCode {
    type Error = TeamError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !value.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(TeamError::FifaCodeFormat(value));
        }
        if value.chars().count() != 3 {
            return Err(TeamError::FifaCodeLength(value));
        }
        Ok(FifaCode(value))
    }
}

/// Type alias for a collection of teams.
///
/// Indexable by [`TeamId`]
pub type Teams = HashMap<TeamId, Team>;

/// External rank (e.g. Fifa or Uefa rank)
#[derive(
    Deserialize,
    Serialize,
    Debug,
    Clone,
    Copy,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::hash::Hash,
    std::cmp::PartialOrd,
    std::cmp::Ord,
    From,
    Into,
)]
pub struct TeamRank(pub u32);

/// Look-up table for Fifa code to iso2 value.
///
/// Taking the lower case first two letters of the Fifa code is a good heuristic for this
/// conversion but for some countries it fails. These failed states are added to this look-up
/// table (when discovered).
const FIFA_CODE_ISO2_MAP: &[(&str, &str)] = &[
    ("ENG", "gb-eng"),
    ("POL", "pl"),
    ("POR", "pt"),
    ("SLO", "sk"),
    ("SUI", "ch"),
    ("SWE", "se"),
    ("TUR", "tr"),
    ("UKR", "ua"),
    ("WAL", "gb-wls"),
];

/// Heuristic mapping for 'Fifa code -> ISO2' codes.
///
/// Look-up, or the first two letters of the Fifa code in lower case
///
/// # Panics
///
/// Never panics, [`FifaCode`] is guaranteed to have exactly three (upper case ASCII) chars.
/// The function assumes that it can access the two first chars, and so can never fail.
impl From<&FifaCode> for Iso2 {
    fn from(fifa_code: &FifaCode) -> Iso2 {
        if let Some((_, iso2)) = FIFA_CODE_ISO2_MAP
            .iter()
            .find(|(code, _)| *code == fifa_code.0)
        {
            Iso2::from(String::from(*iso2))
        } else {
            let lower_name = fifa_code.0.to_ascii_lowercase();
            let mut chars = lower_name.chars();
            Iso2((0..2).fold(String::new(), |mut acc, _| {
                acc.push(chars.next().unwrap());
                acc
            }))
        }
    }
}

/// Team error type
#[derive(Error, Debug, Clone)]
pub enum TeamError {
    #[error("Fifa code length should be a trigram, got '{0}'")]
    FifaCodeLength(String),
    #[error("Expected all upper case ASCII Fifa code, got '{0}'")]
    FifaCodeFormat(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::matches;
    #[test]
    fn deserialize() {
        let data = r#"
        {
            "id": 0,
            "name": "Sweden",
            "iso2": "se",
            "fifaCode": "SWE",
            "rank": 14
        }"#;
        let parsed_team: Team = serde_json::from_str(data).unwrap();
        let true_team = Team::try_new(0.into(), "Sweden", "SWE", 14.into())
            .expect("Team creation should not fail.");
        assert_eq!(parsed_team, true_team);
    }

    #[test]
    fn fifa_code_parse() {
        let ok = FifaCode::try_from(String::from("POL")).expect("Fifa code parse should not fail.");
        assert_eq!(ok, FifaCode(String::from("POL")));

        let too_long = FifaCode::try_from(String::from("POLA"));
        assert!(matches!(too_long, Err(TeamError::FifaCodeLength(..))));

        let too_short = FifaCode::try_from(String::from("PO"));
        assert!(matches!(too_short, Err(TeamError::FifaCodeLength(..))));

        let not_all_upper_case = FifaCode::try_from(String::from("Pol"));
        assert!(matches!(
            not_all_upper_case,
            Err(TeamError::FifaCodeFormat(..))
        ));

        let non_ascii = FifaCode::try_from(String::from("PÖL"));
        assert!(matches!(non_ascii, Err(TeamError::FifaCodeFormat(..))));
    }
}
