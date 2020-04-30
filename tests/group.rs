use wednesday_world_cup::data::lsv;
use wednesday_world_cup::group::order;
use wednesday_world_cup::group::GroupId;
use wednesday_world_cup::team::TeamId;
use wednesday_world_cup::utils::file_io;
#[test]
fn group_ordering() {
    let data_json = file_io::read_json_file_to_str("tests/data/wc-2018.json")
        .expect("Could not read from file");
    let data: lsv::Data = serde_json::from_str(&data_json).expect("JSON format error.");
    let groups = lsv::try_groups_from_data(&data).expect("Could not parse groups from data");

    let id = GroupId('a');
    let group = groups
        .get(&id)
        .expect(&format!("No group '{:?}' in parsed groups", &id));
    assert_eq!(group.winner(order::fifa_2018_rules), TeamId(4));

    let id = GroupId('d');
    let group = groups
        .get(&id)
        .expect(&format!("No group '{:?}' in parsed groups", &id));
    assert_eq!(group.runner_up(order::fifa_2018_rules), TeamId(13));
}
