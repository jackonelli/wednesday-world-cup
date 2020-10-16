use crate::schema::games;
use serde::Serialize;

#[derive(Debug, Serialize, Queryable)]
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

//#[derive(Insertable)]
//#[table_name = "games"]
//pub struct NewGame {
//    pub type_: String,
//    pub home_team: i32,
//    pub away_team: i32,
//    pub home_result: Option<i32>,
//    pub away_result: Option<i32>,
//    pub home_penalty: Option<i32>,
//    pub away_penalty: Option<i32>,
//    pub home_fair_play: Option<i32>,
//    pub away_fair_play: Option<i32>,
//}
