use std::collections::HashMap;
use std::convert::TryInto;
use wwc_core::group::order::fifa_2018;
use wwc_core::group::stats::{TableStats, UnaryStat};
use wwc_core::group::GroupId;
use wwc_core::team::{Team, TeamId};
use wwc_data::file_io;
use wwc_data::lsv;

fn main() {
    let rules = fifa_2018();
    let data_json = file_io::read_json_file_to_str("data/tests/data/wc-2018.json")
        .expect("Could not read from file");
    let data: lsv::Data = serde_json::from_str(&data_json).expect("JSON format error.");

    let teams: HashMap<TeamId, Team> = data
        .teams
        .clone()
        .into_iter()
        .map(|team| (team.id, team.try_into().unwrap()))
        .collect();

    let groups = lsv::try_groups_from_data(&data).expect("Could not parse groups from data");

    let group_h = groups.get(&GroupId('h')).unwrap();
    let rank = group_h.rank_teams(&rules);
    let table = TableStats::team_stats(group_h);

    println!("Team\tp\t+/-\tg");
    println!("-------------------------");
    for id in rank {
        println!("{}\t{}", teams.get(&id).unwrap(), table.get(&id).unwrap());
    }
}
