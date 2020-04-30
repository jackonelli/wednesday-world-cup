use std::fs::File;
use std::io::Read;
use wednesday_world_cup::data::Data;

#[test]
fn teams_from_full_data() {
    let mut file = File::open("tests/data/wc-2018.json").expect("File not found.");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Error reading file contents to string.");
    let data: Data = serde_json::from_str(&data).expect("JSON format error.");
    assert_eq!(data.teams.len(), 32);
}
