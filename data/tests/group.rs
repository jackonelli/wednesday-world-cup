use wwc_core::group::order::fifa_2018;
use wwc_data::lsv::{get_data, Fifa2018Data, LsvData};

#[test]
fn group_ordering() {
    let rules = fifa_2018();
    let data: Fifa2018Data = get_data("tests/data/wc-2018.json").unwrap();

    let groups = data.try_groups().unwrap();

    for (id, true_winner) in data.group_winners() {
        let group = groups
            .get(&id)
            .unwrap_or_else(|| panic!("No group winner '{:?}' in parsed groups", &id));
        assert_eq!(group.winner(&rules), true_winner);
    }

    for (id, true_runner_up) in data.group_runner_ups() {
        let group = groups
            .get(&id)
            .unwrap_or_else(|| panic!("No group runner up '{:?}' in parsed groups", &id));
        assert_eq!(group.runner_up(&rules), true_runner_up);
    }
}
