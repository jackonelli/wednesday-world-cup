use crate::schema::{games, group_game_map, preds, teams};
use serde::Serialize;
use std::convert::{TryFrom, TryInto};
use wwc_core::fair_play::FairPlayScore;
use wwc_core::game::GameId;
use wwc_core::group::game::{PlayedGroupGame, Score, UnplayedGroupGame};
use wwc_core::team::{FifaCode, Iso2, TeamId, TeamName, TeamRank};

#[derive(Debug, Serialize, Queryable, Identifiable)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub fifa_code: String,
    pub iso2: String,
    pub rank_: i32,
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

impl From<Team> for wwc_core::Team {
    fn from(db_team: Team) -> wwc_core::Team {
        let id = TeamId(u8::try_from(db_team.id).unwrap());
        let name = TeamName::from(db_team.name);
        let fifa_code = FifaCode::from(db_team.fifa_code);
        let iso2 = Iso2::from(db_team.iso2);
        let rank = TeamRank(u8::try_from(db_team.rank_).unwrap());
        wwc_core::Team {
            id,
            name,
            fifa_code,
            iso2,
            rank,
        }
    }
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

impl<'a> From<&'a UnplayedGroupGame> for NewGame<'a> {
    fn from(game: &'a UnplayedGroupGame) -> Self {
        NewGame {
            id: u32::from(game.id).try_into().unwrap_or_else(|err| {
                panic!(
                    "Unplayed group game id conversion, game.id={}. {}",
                    game.id, err
                )
            }),
            type_: "group",
            home_team: u8::from(game.home).into(),
            away_team: u8::from(game.away).into(),
            home_result: None,
            away_result: None,
            home_penalty: None,
            away_penalty: None,
            home_fair_play: None,
            away_fair_play: None,
            played: false,
        }
    }
}

impl<'a> From<&'a PlayedGroupGame> for NewGame<'a> {
    fn from(game: &'a PlayedGroupGame) -> Self {
        NewGame {
            id: u32::from(game.id).try_into().unwrap_or_else(|err| {
                panic!(
                    "Played group game id conversion, game.id={}. {}",
                    game.id, err
                )
            }),
            type_: "group",
            home_team: u8::from(game.home).into(),
            away_team: u8::from(game.away).into(),
            home_result: Some(u8::from(game.score.home).into()),
            away_result: Some(u8::from(game.score.away).into()),
            home_penalty: None,
            away_penalty: None,
            // TODO: FairPlay --> FairPlayScore
            //home_fair_play: Some(u8::from(game.fair_play.home).into()),
            //away_fair_play: Some(u8::from(game.fair_play.away).into()),
            home_fair_play: None,
            away_fair_play: None,
            played: true,
        }
    }
}

impl From<Game> for PlayedGroupGame {
    fn from(game: Game) -> Self {
        PlayedGroupGame {
            id: GameId::from(u32::try_from(game.id).unwrap()),
            home: TeamId::from(u8::try_from(game.home_team).unwrap()),
            away: TeamId::from(u8::try_from(game.away_team).unwrap()),
            score: Score::from((
                u8::try_from(game.home_result.unwrap()).unwrap(),
                u8::try_from(game.away_result.unwrap()).unwrap(),
            )),
            fair_play: FairPlayScore::default(),
            date: wwc_core::Date::mock(),
        }
    }
}

impl From<Game> for UnplayedGroupGame {
    fn from(game: Game) -> Self {
        UnplayedGroupGame {
            id: GameId::from(u32::try_from(game.id).unwrap()),
            home: TeamId::from(u8::try_from(game.home_team).unwrap()),
            away: TeamId::from(u8::try_from(game.away_team).unwrap()),
            date: wwc_core::Date::mock(),
        }
    }
}

#[derive(Debug, Serialize, Queryable, Associations, Identifiable)]
#[primary_key(id)]
#[table_name = "group_game_map"]
#[belongs_to(parent = "Game", foreign_key = "id")]
pub struct GroupGameMap {
    pub id: i32,
    pub group_id_: String,
}

#[derive(Insertable)]
#[table_name = "group_game_map"]
pub struct NewGroupGameMap<'a> {
    pub id: i32,
    pub group_id_: &'a str,
}

#[derive(Debug, Serialize, Queryable, Associations, Identifiable)]
#[belongs_to(parent = "Game")]
pub struct Pred {
    pub id: i32,
    pub game_id: i32,
    pub player: String,
    pub home_result: i32,
    pub away_result: i32,
}

#[derive(Insertable)]
#[table_name = "preds"]
pub struct NewPred<'a> {
    pub id: i32,
    pub player: &'a str,
    pub home_result: i32,
    pub away_result: i32,
}
