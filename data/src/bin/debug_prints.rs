use std::collections::HashMap;
use wwc_core::group::game::PlayedGroupGame;
use wwc_core::group::order::fifa_2018;
use wwc_core::group::stats::{TableStats, UnaryStat};
use wwc_core::group::{GroupId, GroupOrder};
use wwc_core::team::{TeamId, Teams};
use wwc_data::lsv::euro_2021 as euro_2021_data;
use wwc_data::lsv::fifa_2018 as fifa_2018_data;
use wwc_data::lsv::LsvData;
use wwc_data::lsv::LsvParseError;

fn main() -> Result<(), LsvParseError> {
    let rules = fifa_2018();
    let data = euro_2021_data::Euro2021Data::try_data_from_file("data/tests/data/euro-2021.json")?;

    let teams: Teams = data.try_teams()?;

    let groups = data.try_groups()?;

    for (id, group) in groups {
        let rank = group.rank_teams(&rules);
        let table = TableStats::team_stats(&group);
        print_group(id, rank, &teams, table);
    }
    Ok(())
}

fn _print_game(game: &PlayedGroupGame, teams: &Teams) {
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
