#![forbid(unsafe_code)]
use itertools::Itertools;
use structopt::StructOpt;
use thiserror::Error;
use wwc_core::error::WwcError;
use wwc_core::game::GameId;
use wwc_core::group::{Group, GroupId};
use wwc_core::team::Team;
use wwc_data::lsv::get_data;
use wwc_data::lsv::LsvParseError;
use wwc_data::lsv::{Euro2020Data, Fifa2018Data, LsvData};

type Tournament = Fifa2018Data;
const DATA_PATH: &str = "data/lsv_data/complete-fifa-2018.json";

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
    let teams = get_data::<Tournament>(DATA_PATH)?
        .try_teams()?
        .values()
        .cloned()
        .collect::<Vec<Team>>();
    Ok(wwc_db::insert_teams(&teams)?)
}

fn add_games() -> Result<(), CliError> {
    let groups = get_data::<Tournament>(DATA_PATH)?
        .try_groups()?
        .values()
        .cloned()
        .collect::<Vec<Group>>();

    let unplayed_games: Vec<_> = groups
        .iter()
        .flat_map(|group| group.unplayed_games())
        .cloned()
        .collect();
    wwc_db::insert_games(&unplayed_games)?;
    let played_games: Vec<_> = groups
        .iter()
        .flat_map(|group| group.played_games())
        .cloned()
        .collect();
    wwc_db::insert_games(&played_games)?;
    Ok(())
}

fn add_groups() -> Result<(), CliError> {
    let groups = get_data::<Tournament>(DATA_PATH)?.try_groups()?;

    let group_games: Vec<(GroupId, GameId)> = groups
        .iter()
        .flat_map(move |(id, group)| {
            group
                .played_games()
                .map(move |game| (*id, game.id))
                .chain(group.unplayed_games().map(move |game| (*id, game.id)))
        })
        .collect();
    wwc_db::insert_group_game_mappings(&group_games)?;
    Ok(())
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
    println!("TEAMS:");
    teams.for_each(|team| println!("{}", team));
    println!();
    Ok(())
}

fn list_games() -> Result<(), CliError> {
    let games = wwc_db::get_games()?;
    println!("GAMES:");
    games.iter().for_each(|game| println!("{:?}", game));
    println!();
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
