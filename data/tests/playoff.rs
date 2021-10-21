use wwc_core::{
    group::order::{euro_2020, euro_2020_third_place, UefaRanking},
    playoff::{game::PlayoffGame, Round},
};
use wwc_data::lsv::{
    euro_2020::playoff::{ParsePlayoff, ParsePlayoffGame},
    get_data, Euro2020Data, LsvData,
};

fn setup() -> (Euro2020Data, Round, ParsePlayoff) {
    let data: Euro2020Data = get_data("tests/data/complete-euro-2020.json").unwrap();

    let groups = data.try_groups().unwrap();
    let teams = data.try_teams().unwrap();
    let ranking = UefaRanking::try_new(
        &groups,
        teams.iter().map(|(id, team)| (*id, team.rank)).collect(),
    )
    .expect("Failed to compile ranking");
    let trans = data.try_playoff_transitions().unwrap();

    let group_rules = euro_2020(ranking.clone());
    let third_place_rules = euro_2020_third_place(ranking);

    let computed_first_round =
        Round::first_round_from_group_stage(&groups, &trans, &group_rules, &third_place_rules);
    let parsed_first_round = data.playoff.clone();
    (data, computed_first_round, parsed_first_round)
}

#[test]
fn id_check() {
    let (_, mut round, parsed_playoff) = setup();

    let mut parsed_first_round = parsed_playoff
        .games()
        .cloned()
        .collect::<Vec<ParsePlayoffGame>>();

    // Sort by id to ensure the correct games are compared
    round.games.sort_by_key(|game| game.id);
    parsed_first_round.sort_by_key(|game| game.id);

    // Id check
    round
        .games
        .iter()
        .map(|game| game.id)
        .zip(parsed_first_round.iter().map(|game| game.id))
        .for_each(|(computed, parsed)| assert_eq!(computed, parsed));
}

#[test]
fn team_to_game_assignment() {
    let (data, mut round, parsed_playoff) = setup();
    let team_map = data.team_map;
    let mut parsed_first_round = parsed_playoff
        .games()
        .map(|game| {
            PlayoffGame::new(
                game.id,
                *team_map.get(&game.home_team.clone().unwrap()).unwrap(),
                *team_map.get(&game.away_team.clone().unwrap()).unwrap(),
            )
        })
        .collect::<Vec<PlayoffGame>>();

    // Sort by id to ensure the correct games are compared
    parsed_first_round.sort_by_key(|game| game.id);
    round.games.sort_by_key(|game| game.id);

    round
        .games
        .iter()
        .map(|game| game.home)
        .zip(parsed_first_round.iter().map(|game| game.home))
        .for_each(|(computed, parsed)| assert_eq!(computed, parsed));
}
