use crate::schema::{games, teams};
use serde::Serialize;
use std::convert::TryFrom;
use wwc_core::team::{FifaCode, Iso2, Rank, TeamId, TeamName};

#[derive(Debug, Serialize, Queryable, Identifiable)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub fifa_code: String,
    pub iso2: String,
    pub rank_: i32,
}

impl Into<wwc_core::Team> for Team {
    fn into(self) -> wwc_core::Team {
        let id = TeamId(u8::try_from(self.id).unwrap());
        let name = TeamName::from(self.name);
        let fifa_code = FifaCode::from(self.fifa_code);
        let iso2 = Iso2::from(self.iso2);
        let rank = Rank(u8::try_from(self.rank_).unwrap());
        wwc_core::Team {
            id,
            name,
            fifa_code,
            iso2,
            rank,
        }
    }
}

#[derive(Insertable)]
#[table_name = "teams"]
pub struct NewTeam<'a> {
    pub id: i32,
    pub name: &'a str,
    pub fifa_code: &'a str,
    pub iso2: &'a str,
    pub rank_: i32,
}

#[derive(Debug, Serialize, Queryable, Associations, Identifiable)]
#[belongs_to(parent = "Team", foreign_key = "id")]
pub struct Game {
    pub id: i32,
    pub type_: String,
    pub home_team: i32,
    pub away_team: i32,
    pub home_result: Option<i32>,
    pub away_result: Option<i32>,
    pub home_penalty: Option<i32>,
    pub away_penalty: Option<i32>,
    pub home_fair_play: Option<i32>,
    pub away_fair_play: Option<i32>,
    pub played: bool,
}

#[derive(Insertable)]
#[table_name = "games"]
pub struct NewGame<'a> {
    pub id: i32,
    pub type_: &'a str,
    pub home_team: i32,
    pub away_team: i32,
    pub home_result: Option<i32>,
    pub away_result: Option<i32>,
    pub home_penalty: Option<i32>,
    pub away_penalty: Option<i32>,
    pub home_fair_play: Option<i32>,
    pub away_fair_play: Option<i32>,
    pub played: bool,
}
