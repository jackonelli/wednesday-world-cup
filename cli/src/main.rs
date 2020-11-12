#![forbid(unsafe_code)]
use itertools::Itertools;
use std::convert::TryInto;
use structopt::StructOpt;
use wwc_core::group::game::GroupGameId;
use wwc_core::group::{Group, GroupError, GroupId, Groups};
use wwc_core::team::Teams;
use wwc_data::lsv::lsv_data_from_file;

fn main() {
    let opt = Opt::from_args();
    match opt {
        Opt::Add(table) => match table {
            Table::Teams => add_teams(),
            Table::Games => add_games(),
            Table::GroupGameMaps => add_groups(),
            Table::All => {
                add_teams();
                add_games();
                add_groups()
            }
        },
        Opt::List(table) => match table {
            Table::Teams => list_teams(),
            Table::Games => list_games(),
            Table::GroupGameMaps => list_group_maps(),
            Table::All => {
                list_teams();
                list_games();
                list_group_maps()
            }
        },
        Opt::Clear(table) => match table {
            Table::Teams => wwc_db::clear_teams(),
            Table::Games => wwc_db::clear_games(),
            Table::GroupGameMaps => wwc_db::clear_group_game_maps(),
            Table::All => {
                wwc_db::clear_teams();
                wwc_db::clear_games();
                wwc_db::clear_group_game_maps();
            }
        },
    }
}

fn list_teams() {
    let teams = wwc_db::get_teams();
    teams.for_each(|team| println!("{:?}", team));
}

fn list_games() {
    let games = wwc_db::get_games();
    games.iter().for_each(|game| println!("{:?}", game));
}

fn list_group_maps() {
    let group_game_maps = wwc_db::get_group_game_maps();
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
}

fn add_teams() {
    let data = lsv_data_from_file("data/tests/data/wc-2018.json");

    let teams: Teams = data
        .teams
        .into_iter()
        .map(|team| (team.id, team.try_into().unwrap()))
        .collect();
    teams.iter().for_each(|(_, team)| wwc_db::insert_team(team));
}

fn add_games() {
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
    let groups = groups.expect("Could not parse groups");
    groups
        .iter()
        .flat_map(|group| group.unplayed_games())
        .for_each(wwc_db::insert_game);
    groups
        .iter()
        .flat_map(|group| group.played_games())
        .for_each(wwc_db::insert_game);
}

fn add_groups() {
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
    let group_games: Vec<(GroupId, GroupGameId)> = groups
        .iter()
        .flat_map(move |(id, group)| {
            group
                .played_games()
                .map(move |game| (*id, game.id))
                .chain(group.unplayed_games().map(move |game| (*id, game.id)))
        })
        .collect();
    group_games
        .iter()
        .for_each(|x| wwc_db::insert_group_game_mapping(*x));
}

#[derive(Debug, StructOpt)]
#[structopt(name = "bryggio-cli", about = "cli usage")]
pub enum Opt {
    #[structopt(name = "add")]
    Add(Table),
    #[structopt(name = "list")]
    List(Table),
    #[structopt(name = "clear")]
    Clear(Table),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "bryggio-cli", about = "cli usage")]
pub enum Table {
    #[structopt(name = "teams")]
    Teams,
    #[structopt(name = "games")]
    Games,
    #[structopt(name = "group-game-maps")]
    GroupGameMaps,
    #[structopt(name = "all")]
    All,
}
