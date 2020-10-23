#![forbid(unsafe_code)]
use std::convert::TryFrom;
use std::convert::TryInto;
use structopt::StructOpt;
use wwc_core::group::game::GroupGameId;
use wwc_core::group::{Group, GroupError, GroupId, Groups};
use wwc_core::team::{Team, Teams};
use wwc_data::lsv::lsv_data_from_file;

fn main() {
    let opt = Opt::from_args();
    match opt {
        Opt::AddTeams => add_teams(),
        Opt::ListTeams => list_teams(),
        Opt::AddGroups => add_groups(),
        Opt::AddGames => add_games(), //add_games(),
        Opt::ListGames => list_games(),
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

fn add_teams() {
    let data = lsv_data_from_file("data/tests/data/wc-2018.json");

    let teams: Teams = data
        .teams
        .clone()
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
        .for_each(|x| wwc_db::insert_game(x));
    groups
        .iter()
        .flat_map(|group| group.played_games())
        .for_each(|x| wwc_db::insert_game(x));
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
    #[structopt(name = "add-teams")]
    AddTeams,
    #[structopt(name = "list-teams")]
    ListTeams,
    #[structopt(name = "add-games")]
    AddGames,
    #[structopt(name = "list-games")]
    ListGames,
    #[structopt(name = "add-groups")]
    AddGroups,
}
