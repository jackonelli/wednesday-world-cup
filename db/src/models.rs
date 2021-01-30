use crate::schema::{games, group_game_map, players, preds, teams};
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
        let id = TeamId(u32::try_from(db_team.id).unwrap());
        let name = TeamName::from(db_team.name);
        let fifa_code = FifaCode::from(db_team.fifa_code);
        let iso2 = Iso2::from(db_team.iso2);
        let rank = TeamRank(u32::try_from(db_team.rank_).unwrap());
        wwc_core::Team {
            id,
            name,
            fifa_code,
            iso2,
            rank,
        }
    }
}

impl<'a> From<&'a wwc_core::Team> for NewTeam<'a> {
    fn from(team: &'a wwc_core::Team) -> NewTeam<'a> {
        NewTeam {
            id: u32::from(team.id).try_into().expect("team id u32 -> i32"),
            name: team.name.as_ref(),
            fifa_code: team.fifa_code.as_ref(),
            iso2: team.iso2.as_ref(),
            rank_: u32::from(team.rank).try_into().expect("team id u32 -> i32"),
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
            home_team: u32::from(game.home).try_into().expect("team id u32 -> i32"),
            away_team: u32::from(game.away).try_into().expect("team id u32 -> i32"),
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
            home_team: u32::from(game.home).try_into().expect("team id u32 -> i32"),
            away_team: u32::from(game.away).try_into().expect("team id u32 -> i32"),
            home_result: Some(
                u32::from(game.score.home)
                    .try_into()
                    .expect("result u32 -> i32"),
            ),
            away_result: Some(
                u32::from(game.score.away)
                    .try_into()
                    .expect("result u32 -> i32"),
            ),
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
            home: TeamId::from(u32::try_from(game.home_team).unwrap()),
            away: TeamId::from(u32::try_from(game.away_team).unwrap()),
            score: Score::from((
                u32::try_from(game.home_result.unwrap()).unwrap(),
                u32::try_from(game.away_result.unwrap()).unwrap(),
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
            home: TeamId::from(u32::try_from(game.home_team).unwrap()),
            away: TeamId::from(u32::try_from(game.away_team).unwrap()),
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
#[belongs_to(parent = "Player")]
pub struct Pred {
    pub id: i32,
    pub player_id: i32,
    pub game_id: i32,
    pub home_result: i32,
    pub away_result: i32,
}

#[derive(Insertable)]
#[table_name = "preds"]
pub struct NewPred {
    pub id: i32,
    pub player_id: i32,
    pub home_result: i32,
    pub away_result: i32,
}

#[derive(Debug, Serialize, Queryable, Associations, Identifiable)]
pub struct Player {
    pub id: i32,
    pub name: String,
}

#[derive(Insertable)]
#[table_name = "players"]
pub struct NewPlayer<'a> {
    pub name: &'a str,
}
