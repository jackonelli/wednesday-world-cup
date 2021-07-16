//! Team
use derive_more::{AsRef, Display, From, Into};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
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

pub type Teams = HashMap<TeamId, Team>;

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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Team {
    pub id: TeamId,
    pub name: TeamName,
    #[serde(rename = "fifaCode")]
    pub fifa_code: FifaCode,
    pub iso2: Iso2,
    pub rank: TeamRank,
}

impl Team {
    pub fn new<TN: AsRef<str> + ?Sized>(
        id: TeamId,
        name: &TN,
        fifa_code: &str,
        iso2: &str,
        rank: TeamRank,
    ) -> Self {
        Team {
            id,
            name: TeamName(String::from(name.as_ref())),
            fifa_code: FifaCode(String::from(fifa_code)),
            iso2: Iso2(String::from(iso2)),
            rank,
        }
    }
}

impl std::fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.fifa_code)
    }
}

#[derive(Display, Debug, Clone, AsRef, From, Into, Deserialize, Serialize, PartialEq)]
#[as_ref(forward)]
pub struct TeamName(pub(crate) String);
#[derive(Display, Debug, Clone, AsRef, From, Into, Deserialize, Serialize, PartialEq)]
#[as_ref(forward)]
pub struct FifaCode(String);
#[derive(Display, Debug, Clone, AsRef, From, Into, Deserialize, Serialize, PartialEq)]
#[as_ref(forward)]
pub struct Iso2(String);

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

/// Ad hoc mapping for 'Fifa code -> ISO2' codes
/// Look up, or the first two letters of the Fifa code in lowercase
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
#[cfg(test)]
mod tests {
    use super::*;
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
        let true_team = Team::new(0.into(), "Sweden", "SWE", "se", 14.into());
        assert_eq!(parsed_team, true_team);
    }
}
