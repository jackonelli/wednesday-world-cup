use wwc_data::lsv::{get_data, Euro2020Data, Fifa2018Data, LsvData};

#[test]
fn fifa_2018_teams_from_full_data() {
    let data: Fifa2018Data = get_data("tests/data/blank-fifa-2018.json").unwrap();
    assert_eq!(data.try_teams().unwrap().len(), 32);
}

#[test]
fn euro_2020_teams_from_full_data() {
    let data: Euro2020Data = get_data("tests/data/blank-euro-2020.json").unwrap();
    assert_eq!(data.try_teams().unwrap().len(), 24);
}
