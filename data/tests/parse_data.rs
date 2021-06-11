use wwc_data::lsv::{get_data, Fifa2018Data, LsvData};

#[test]
fn teams_from_full_data() {
    let data: Fifa2018Data = get_data("tests/data/wc-2018.json").unwrap();
    assert_eq!(data.try_teams().unwrap().len(), 32);
}
