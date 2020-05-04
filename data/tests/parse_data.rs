use wwc_data::file_io;
use wwc_data::lsv;

#[test]
fn teams_from_full_data() {
    let data_json = file_io::read_json_file_to_str("tests/data/wc-2018.json")
        .expect("Could not read from file");
    let data: lsv::Data = serde_json::from_str(&data_json).expect("JSON format error.");
    assert_eq!(data.teams.len(), 32);
}
