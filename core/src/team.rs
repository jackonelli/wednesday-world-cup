//! Team
use derive_more::{Display, From};
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
)]
pub struct TeamId(pub u8);

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
)]
pub struct Rank(pub u8);

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Team {
    pub id: TeamId,
    pub name: TeamName,
    #[serde(rename = "fifaCode")]
    pub fifa_code: FifaCode,
    pub iso2: Iso2,
    rank: Rank,
}

impl Team {
    pub fn new(id: TeamId, name: &str, fifa_code: &str, iso2: &str, rank: Rank) -> Self {
        Team {
            id,
            name: TeamName(String::from(name)),
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

#[derive(Display, Debug, Deserialize, Serialize, PartialEq)]
pub struct TeamName(pub(crate) String);
#[derive(Display, Debug, Deserialize, Serialize, PartialEq)]
pub struct FifaCode(String);
#[derive(Display, Debug, Deserialize, Serialize, PartialEq)]
pub struct Iso2(String);

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
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
