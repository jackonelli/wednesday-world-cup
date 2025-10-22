//! Database models (structs that map to database rows)

use crate::DbError;
use sqlx::FromRow;
use wwc_core::error::WwcError;
use wwc_core::fair_play::FairPlayScore;
use wwc_core::game::{GameId, GoalCount};
use wwc_core::group::game::{GroupGameScore, PlayedGroupGame, UnplayedGroupGame};
use wwc_core::player::Prediction;
use wwc_core::team::{FifaCode, TeamId, TeamName, TeamRank};

#[derive(Debug, FromRow)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub fifa_code: String,
    pub rank_: i32,
}

impl From<Team> for wwc_core::Team {
    fn from(db_team: Team) -> wwc_core::Team {
        let id = TeamId(u32::try_from(db_team.id).unwrap());
        let name = TeamName::from(db_team.name);
        let fifa_code = FifaCode::try_from(db_team.fifa_code).unwrap();
        let rank = TeamRank(u32::try_from(db_team.rank_).unwrap());
        wwc_core::Team {
            id,
            name,
            fifa_code,
            rank,
        }
    }
}

#[derive(Debug, FromRow)]
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

impl TryFrom<Game> for PlayedGroupGame {
    type Error = DbError;
    fn try_from(game: Game) -> Result<Self, Self::Error> {
        Ok(UnplayedGroupGame::try_new(
            u32::try_from(game.id).unwrap(),
            u32::try_from(game.home_team).unwrap(),
            u32::try_from(game.away_team).unwrap(),
            wwc_core::Date::mock(),
        )
        .map_err(WwcError::from)
        .map_err(DbError::from)?
        .play(
            GroupGameScore::from((
                GoalCount::try_from(u32::try_from(game.home_result.unwrap()).unwrap()).unwrap(),
                GoalCount::try_from(u32::try_from(game.away_result.unwrap()).unwrap()).unwrap(),
            )),
            FairPlayScore::default(),
        ))
    }
}

impl TryFrom<Game> for UnplayedGroupGame {
    type Error = DbError;
    fn try_from(game: Game) -> Result<Self, Self::Error> {
        UnplayedGroupGame::try_new(
            u32::try_from(game.id).unwrap(),
            u32::try_from(game.home_team).unwrap(),
            u32::try_from(game.away_team).unwrap(),
            wwc_core::Date::mock(),
        )
        .map_err(WwcError::from)
        .map_err(DbError::from)
    }
}

#[derive(Debug, FromRow)]
pub struct GroupGameMap {
    pub id: i32,
    pub group_id_: String,
}

#[derive(Debug, FromRow)]
pub struct PlayoffTeamSourceRow {
    pub game_id: i32,
    pub home_source_type: String,
    pub home_group_id: Option<String>,
    pub home_outcome: Option<String>,
    pub home_third_place_groups: Option<String>,
    pub home_source_game_id: Option<i32>,
    pub away_source_type: String,
    pub away_group_id: Option<String>,
    pub away_outcome: Option<String>,
    pub away_third_place_groups: Option<String>,
    pub away_source_game_id: Option<i32>,
}

#[derive(Debug, FromRow)]
pub struct Pred {
    pub id: i32,
    pub player_id: i32,
    pub game_id: i32,
    pub home_result: i32,
    pub away_result: i32,
}

impl From<Pred> for Prediction {
    fn from(pred: Pred) -> Prediction {
        let score = GroupGameScore::from((
            GoalCount::try_from(u32::try_from(pred.home_result).unwrap()).unwrap(),
            GoalCount::try_from(u32::try_from(pred.away_result).unwrap()).unwrap(),
        ));
        Prediction(GameId::from(u32::try_from(pred.game_id).unwrap()), score)
    }
}

#[derive(Debug, FromRow)]
pub struct Player {
    pub id: i32,
    pub name: String,
}
