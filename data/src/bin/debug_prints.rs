use std::collections::HashMap;
use std::convert::TryInto;
use wwc_core::group::game::PlayedGroupGame;
use wwc_core::group::order::fifa_2018;
use wwc_core::group::stats::{TableStats, UnaryStat};
use wwc_core::group::{GroupId, GroupOrder};
use wwc_core::team::{Team, TeamId, Teams};
use wwc_data::lsv;

fn main() {
    let rules = fifa_2018();
    let data = lsv::lsv_data_from_file("data/tests/data/wc-2018.json");

    let teams: HashMap<TeamId, Team> = data
        .teams
        .clone()
        .into_iter()
        .map(|team| (team.id, team.try_into().unwrap()))
        .collect();

    let groups = lsv::try_groups_from_data(&data).expect("Could not parse groups from data");

    for (id, group) in groups {
        let rank = group.rank_teams(&rules);
        let table = TableStats::team_stats(&group);
        print_group(id, rank, &teams, table);
    }
}

fn print_game(game: &PlayedGroupGame, teams: &Teams) {
    let home = teams.get(&game.home).unwrap();
    let away = teams.get(&game.away).unwrap();
    println!(
        "{} {} - {} {}",
        home, game.score.home, game.score.away, away
    );
}

fn print_group(id: GroupId, rank: GroupOrder, teams: &Teams, table: HashMap<TeamId, TableStats>) {
    println!("Group {}", id);
    println!("*************************");
    println!("Team\tp\t+/-\tg");
    println!("-------------------------");
    for id in rank {
        println!("{}\t{}", teams.get(&id).unwrap(), table.get(&id).unwrap());
    }
    println!("*************************");
}
