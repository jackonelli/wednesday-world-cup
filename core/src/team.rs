use derive_more::{Display, From};
use serde::{Deserialize, Serialize};
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
    name: String,
    #[serde(rename = "fifaCode")]
    fifa_code: String,
    iso2: String,
    rank: Rank,
}

impl Team {
    pub fn new(id: TeamId, name: &str, fifa_code: &str, iso2: &str, rank: Rank) -> Self {
        Team {
            id,
            name: String::from(name),
            fifa_code: String::from(fifa_code),
            iso2: String::from(iso2),
            rank,
        }
    }
}

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
        let true_team = Team {
            id: 0.into(),
            name: "Sweden".into(),
            fifa_code: "SWE".into(),
            iso2: "se".into(),
            rank: 14.into(),
        };
        assert_eq!(parsed_team, true_team);
    }
}
