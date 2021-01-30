#![forbid(unsafe_code)]
use itertools::Itertools;
use std::convert::{TryFrom, TryInto};
use structopt::StructOpt;
use thiserror::Error;
use wwc_core::error::WwcError;
use wwc_core::game::GameId;
use wwc_core::group::{Group, GroupError, GroupId, Groups};
use wwc_core::team::{Team, Teams};
use wwc_data::lsv::{lsv_data_from_file, LsvParseError};

fn main() -> Result<(), CliError> {
    let opt = Opt::from_args();
    match opt {
        Opt::Register(new_instance) => match new_instance {
            Instance::Player { name } => register_player(name),
        },
        Opt::Add(table) => match table {
            Table::Players => Ok(()),
            Table::Teams => add_teams(),
            Table::Games => add_games(),
            Table::GroupGameMaps => add_groups(),
            Table::All => {
                add_teams()?;
                add_games()?;
                add_groups()
            }
        },
        Opt::List(table) => match table {
            Table::Players => list_players(),
            Table::Teams => list_teams(),
            Table::Games => list_games(),
            Table::GroupGameMaps => list_group_maps(),
            Table::All => {
                list_players()?;
                list_teams()?;
                list_games()?;
                list_group_maps()
            }
        },
        Opt::Clear(table) => match table {
            Table::Players => Ok(wwc_db::clear_players()?),
            Table::Teams => Ok(wwc_db::clear_teams()?),
            Table::Games => Ok(wwc_db::clear_games()?),
            Table::GroupGameMaps => Ok(wwc_db::clear_group_game_maps()?),
            Table::All => {
                wwc_db::clear_teams()?;
                wwc_db::clear_games()?;
                Ok(wwc_db::clear_group_game_maps()?)
            }
        },
    }
}

fn register_player(name: String) -> Result<(), CliError> {
    Ok(wwc_db::register_player(&name)?)
}

fn add_teams() -> Result<(), CliError> {
    let data = lsv_data_from_file("data/tests/data/wc-2018.json");

    let teams: Result<Teams, CliError> = data
        .teams
        .into_iter()
        .map(|parse_team| Team::try_from(parse_team).map_err(CliError::from))
        .map(|team| team.map(|ok_team| (ok_team.id, ok_team)))
        .collect();

    Ok(teams?
        .iter()
        .try_for_each(|(_, team)| wwc_db::insert_team(team))?)
}

fn add_games() -> Result<(), CliError> {
    let data = lsv_data_from_file("data/tests/data/wc-2018.json");

    let groups: Result<Vec<Group>, GroupError> = data
        .groups
        .into_iter()
        .map(|(_, pg)| {
            // TODO: Do not know why Group::try_from(pg) does not work
            let group: Result<Group, GroupError> = pg.try_into();
            group
        })
        .collect();
    let groups = groups.map_err(WwcError::from)?;
    groups
        .iter()
        .flat_map(|group| group.unplayed_games())
        .try_for_each(wwc_db::insert_game)?;
    groups
        .iter()
        .flat_map(|group| group.played_games())
        .try_for_each(wwc_db::insert_game)?;
    Ok(())
}

fn add_groups() -> Result<(), CliError> {
    let data = lsv_data_from_file("data/tests/data/wc-2018.json");

    let groups: Result<Groups, GroupError> = data
        .groups
        .into_iter()
        .map(|(id, pg)| {
            // TODO: Do not know why Group::try_from(pg) does not work
            let group: Result<Group, GroupError> = pg.try_into();
            match group {
                Ok(group) => Ok((id, group)),
                Err(_) => Err(GroupError::GenericError),
            }
        })
        .collect();
    let groups = groups.expect("Could not parse groups");
    let group_games: Vec<(GroupId, GameId)> = groups
        .iter()
        .flat_map(move |(id, group)| {
            group
                .played_games()
                .map(move |game| (*id, game.id))
                .chain(group.unplayed_games().map(move |game| (*id, game.id)))
        })
        .collect();
    Ok(group_games
        .iter()
        .try_for_each(|x| wwc_db::insert_group_game_mapping(*x))?)
}

fn list_players() -> Result<(), CliError> {
    let players = wwc_db::get_players()?;
    // Very strange bug:
    // players.for_each(|player| println!("{:?}", player));
    for player in players {
        println!("{:?}", player);
    }
    Ok(())
}

fn list_teams() -> Result<(), CliError> {
    let teams = wwc_db::get_teams()?;
    teams.for_each(|team| println!("{:?}", team));
    Ok(())
}

fn list_games() -> Result<(), CliError> {
    let games = wwc_db::get_games()?;
    games.iter().for_each(|game| println!("{:?}", game));
    Ok(())
}

fn list_group_maps() -> Result<(), CliError> {
    let group_game_maps = wwc_db::get_group_game_maps()?;
    group_game_maps
        .map(|(game, group)| (group, game))
        .into_group_map()
        .iter()
        .for_each(|(group_id, games)| {
            println!(
                "{}: {}",
                group_id,
                games
                    .iter()
                    .fold(String::new(), |acc, x| format!("{} {},", acc, x))
            )
        });
    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(name = "bryggio-cli", about = "cli usage")]
pub enum Opt {
    #[structopt(name = "register")]
    Register(Instance),
    #[structopt(name = "add")]
    Add(Table),
    #[structopt(name = "list")]
    List(Table),
    #[structopt(name = "clear")]
    Clear(Table),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "bryggio-cli", about = "cli usage")]
pub enum Instance {
    #[structopt(name = "player")]
    Player { name: String },
}

#[derive(Debug, StructOpt)]
#[structopt(name = "bryggio-cli", about = "cli usage")]
pub enum Table {
    #[structopt(name = "players")]
    Players,
    #[structopt(name = "teams")]
    Teams,
    #[structopt(name = "games")]
    Games,
    #[structopt(name = "group-game-maps")]
    GroupGameMaps,
    #[structopt(name = "all")]
    All,
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Cli: {0}")]
    Db(#[from] wwc_db::DbError),
    #[error("Cli: {0}")]
    WwcCore(#[from] WwcError),
    #[error("Parse: {0}")]
    Parse(#[from] LsvParseError),
}
