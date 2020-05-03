use derive_more::From;
use serde::{Deserialize, Serialize};
#[derive(
    Deserialize,
    Serialize,
    Debug,
    Clone,
    Copy,
    std::cmp::Eq,
    std::cmp::PartialEq,
    std::hash::Hash,
    From,
)]
pub struct TeamId(pub u8);

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Team {
    id: TeamId,
    name: String,
    #[serde(rename = "fifaCode")]
    fifa_code: String,
    iso2: String,
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
            "fifaCode": "SWE"
        }"#;
        let parsed_team: Team = serde_json::from_str(data).unwrap();
        let true_team = Team {
            id: 0.into(),
            name: "Sweden".into(),
            fifa_code: "SWE".into(),
            iso2: "se".into(),
        };
        assert_eq!(parsed_team, true_team);
    }
}
