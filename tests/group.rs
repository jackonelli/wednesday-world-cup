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
    let group_a = groups
        .get(&GroupId('a'))
        .expect("No group 'a' in parsed groups");
    assert_eq!(group_a.winner(order::fifa_2018_rules), TeamId(4));
}
