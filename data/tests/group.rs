use wwc_core::group::order::{UefaRanking, euro_2020_rules, fifa_2018_rules};
use wwc_data::lsv::{Euro2020Data, Fifa2018Data, LsvData, get_data};

#[test]
fn fifa_2018_group_ordering() {
    let rules = fifa_2018_rules();
    let data: Fifa2018Data = get_data("lsv_data/complete-fifa-2018.json").unwrap();

    let groups = data.try_groups().unwrap();

    for (id, true_winner) in data.group_winners() {
        let group = groups
            .get(&id)
            .unwrap_or_else(|| panic!("No group winner '{:?}' in parsed groups", &id));
        assert_eq!(
            group.winner(&rules),
            true_winner.expect("Group winner should be present in the test data")
        );
    }

    for (id, true_runner_up) in data.group_runner_ups() {
        let group = groups
            .get(&id)
            .unwrap_or_else(|| panic!("No group runner up '{:?}' in parsed groups", &id));
        assert_eq!(
            group.runner_up(&rules),
            true_runner_up.expect("Group runner up should be present in the test data")
        );
    }
}

#[test]
fn euro_2020_group_ordering() {
    let data: Euro2020Data = get_data("lsv_data/complete-euro-2020.json").unwrap();

    let groups = data.try_groups().unwrap();
    let teams = data.try_teams().unwrap();
    let ranking = UefaRanking::try_new(
        &groups,
        teams.iter().map(|(id, team)| (*id, team.rank)).collect(),
    )
    .expect("Failed to compile ranking");

    let rules = euro_2020_rules(ranking);
    for (id, true_winner) in data.group_winners() {
        let group = groups
            .get(&id)
            .unwrap_or_else(|| panic!("No group winner '{:?}' in parsed groups", &id));
        assert_eq!(
            teams.get(&group.winner(&rules)).unwrap().fifa_code,
            true_winner.expect("Group winner should be present in the test data")
        );
    }

    for (id, true_runner_up) in data.group_runner_ups() {
        let group = groups
            .get(&id)
            .unwrap_or_else(|| panic!("No group runner up '{:?}' in parsed groups", &id));
        assert_eq!(
            teams.get(&group.runner_up(&rules)).unwrap().fifa_code,
            true_runner_up.expect("Group runner up should be present in the test data")
        );
    }
}
