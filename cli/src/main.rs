#![forbid(unsafe_code)]
use itertools::Itertools;
use structopt::StructOpt;
use thiserror::Error;
use wwc_core::error::WwcError;
use wwc_core::game::GameId;
use wwc_core::group::{Group, GroupId};
use wwc_core::team::Team;
use wwc_data::lsv::LsvParseError;
use wwc_data::lsv::get_data;
use wwc_data::lsv::{Euro2020Data, Fifa2018Data, LsvData};

type Tournament = Fifa2018Data;
const DATA_PATH: &str = "data/lsv_data/complete-fifa-2018.json";

#[tokio::main]
async fn main() -> Result<(), CliError> {
    let pool = wwc_db::create_pool().await?;
    let opt = Opt::from_args();
    match opt {
        Opt::Register(new_instance) => match new_instance {
            Register::Player { name } => register_player(&pool, name).await,
        },
        Opt::Add(table) => match table {
            Table::Players => Ok(()),
            Table::Teams => add_teams(&pool).await,
            Table::Games => add_games(&pool).await,
            Table::GroupGameMaps => add_groups(&pool).await,
            Table::All => {
                add_teams(&pool).await?;
                add_games(&pool).await?;
                add_groups(&pool).await
            }
        },
        Opt::List(table) => match table {
            Table::Players => list_players(&pool).await,
            Table::Teams => list_teams(&pool).await,
            Table::Games => list_games(&pool).await,
            Table::GroupGameMaps => list_group_maps(&pool).await,
            Table::All => {
                list_players(&pool).await?;
                list_teams(&pool).await?;
                list_games(&pool).await?;
                list_group_maps(&pool).await
            }
        },
        Opt::Clear(table) => match table {
            Table::Players => Ok(wwc_db::clear_players(&pool).await?),
            Table::Teams => Ok(wwc_db::clear_teams(&pool).await?),
            Table::Games => Ok(wwc_db::clear_games(&pool).await?),
            Table::GroupGameMaps => Ok(wwc_db::clear_group_game_maps(&pool).await?),
            Table::All => {
                wwc_db::clear_teams(&pool).await?;
                wwc_db::clear_games(&pool).await?;
                Ok(wwc_db::clear_group_game_maps(&pool).await?)
            }
        },
    }
}

async fn register_player(pool: &sqlx::SqlitePool, name: String) -> Result<(), CliError> {
    Ok(wwc_db::register_player(pool, &name).await?)
}

async fn add_teams(pool: &sqlx::SqlitePool) -> Result<(), CliError> {
    let teams = get_data::<Tournament>(DATA_PATH)?
        .try_teams()?
        .values()
        .cloned()
        .collect::<Vec<Team>>();
    Ok(wwc_db::insert_teams(pool, &teams).await?)
}

async fn add_games(pool: &sqlx::SqlitePool) -> Result<(), CliError> {
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
    wwc_db::insert_unplayed_games(pool, &unplayed_games).await?;
    let played_games: Vec<_> = groups
        .iter()
        .flat_map(|group| group.played_games())
        .cloned()
        .collect();
    wwc_db::insert_played_games(pool, &played_games).await?;
    Ok(())
}

async fn add_groups(pool: &sqlx::SqlitePool) -> Result<(), CliError> {
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
    wwc_db::insert_group_game_mappings(pool, &group_games).await?;
    Ok(())
}

async fn list_players(pool: &sqlx::SqlitePool) -> Result<(), CliError> {
    let players = wwc_db::get_players(pool).await?;
    for player in players {
        println!("{:?}", player);
    }
    Ok(())
}

async fn list_teams(pool: &sqlx::SqlitePool) -> Result<(), CliError> {
    let teams = wwc_db::get_teams(pool).await?;
    println!("TEAMS:");
    teams.iter().for_each(|team| println!("{}", team));
    println!();
    Ok(())
}

async fn list_games(pool: &sqlx::SqlitePool) -> Result<(), CliError> {
    let games = wwc_db::get_games(pool).await?;
    println!("GAMES:");
    games.iter().for_each(|game| println!("{:?}", game));
    println!();
    Ok(())
}

async fn list_group_maps(pool: &sqlx::SqlitePool) -> Result<(), CliError> {
    let group_game_maps = wwc_db::get_group_game_maps(pool).await?;
    group_game_maps
        .into_iter()
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
#[structopt(name = "wwc-cli", about = "cli usage")]
pub enum Opt {
    #[structopt(name = "register")]
    Register(Register),
    #[structopt(name = "add")]
    Add(Table),
    #[structopt(name = "list")]
    List(Table),
    #[structopt(name = "clear")]
    Clear(Table),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "wwc-cli-register", about = "Register new player")]
pub enum Register {
    #[structopt(name = "player")]
    Player { name: String },
}

#[derive(Debug, StructOpt)]
#[structopt(name = "wwc-cli-table", about = "Add table to database")]
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
