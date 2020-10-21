#![forbid(unsafe_code)]
use std::convert::TryInto;
use structopt::StructOpt;
use wwc_core::team::{Team, Teams};
use wwc_data::lsv::lsv_data_from_file;

fn main() {
    let opt = Opt::from_args();
    match opt {
        Opt::AddTeams => add_teams(),
        Opt::ListTeams => list_teams(),
        Opt::AddGames => {} //add_games(),
    }
}

fn list_teams() {
    let teams = wwc_db::get_teams();
    teams.for_each(|team| println!("{:?}", team));
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

#[derive(Debug, StructOpt)]
#[structopt(name = "bryggio-cli", about = "cli usage")]
pub enum Opt {
    #[structopt(name = "add-teams")]
    AddTeams,
    #[structopt(name = "list-teams")]
    ListTeams,
    #[structopt(name = "add-games")]
    AddGames,
}
