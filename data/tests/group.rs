use wwc_core::group::order::fifa_2018;
use wwc_core::group::GroupId;
use wwc_core::team::TeamId;
use wwc_data::file_io;
use wwc_data::lsv;
#[test]
fn group_ordering() {
    let rules = fifa_2018();
    let data_json = file_io::read_json_file_to_str("tests/data/wc-2018.json")
        .expect("Could not read from file");
    let data: lsv::Data = serde_json::from_str(&data_json).expect("JSON format error.");
    let groups = lsv::try_groups_from_data(&data).expect("Could not parse groups from data");

    let id = GroupId('a');
    let group = groups
        .get(&id)
        .expect(&format!("No group '{:?}' in parsed groups", &id));
    assert_eq!(group.winner(&rules), TeamId(4));

    let id = GroupId('d');
    let group = groups
        .get(&id)
        .expect(&format!("No group '{:?}' in parsed groups", &id));
    assert_eq!(group.runner_up(&rules), TeamId(13));
}
