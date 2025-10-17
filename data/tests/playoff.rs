use wwc_core::{
    group::order::{
        UefaRanking, euro_2020_rules, euro_2020_third_place_rules, fifa_2018_rules,
        noop_fifa_2018_third_place_rules,
    },
    playoff::{Round, game::PlayoffGame},
};
use wwc_data::lsv::euro_2020::playoff::{
    ParsePlayoff as Euro2020ParsePlayoff, ParsePlayoffGame as Euro2020ParsePlayoffGame,
};
use wwc_data::lsv::fifa_2018::playoff::{
    ParsePlayoff as Fifa2018arsePlayoff, ParsePlayoffGame as Fifa2018arsePlayoffGame,
};
use wwc_data::lsv::{Euro2020Data, Fifa2018Data, LsvData, get_data};

#[test]
fn fifa2018_id_check() {
    let (_, _, mut round, parsed_playoff) = fifa_2018_setup();

    let mut parsed_first_round = parsed_playoff
        .games()
        .cloned()
        .collect::<Vec<Fifa2018arsePlayoffGame>>();

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
fn fifa2018_team_to_game_assignment() {
    let (_, complete_data, mut round, _) = fifa_2018_setup();
    let mut parsed_first_round = complete_data
        .playoff
        .games()
        .map(|game| {
            PlayoffGame::new(
                game.id,
                game.home_team.team_from_finished().unwrap(),
                game.away_team.team_from_finished().unwrap(),
            )
        })
        .collect::<Vec<PlayoffGame>>();

    // Sort by id to ensure the correct games are compared
    parsed_first_round.sort_by_key(|game| game.id);
    round.games.sort_by_key(|game| game.id);

    parsed_first_round
        .iter()
        .zip(round.iter())
        .for_each(|(parsed, comp)| {
            println!(
                "Id: {}=?{}\n\thome:{}=?{}\n\taway:{}=?{}\n\n\n",
                parsed.id,
                comp.id,
                parsed.home.unwrap(),
                comp.home.unwrap(),
                parsed.away.unwrap(),
                comp.away.unwrap()
            )
        });
    round
        .games
        .iter()
        .map(|game| game.home)
        .zip(parsed_first_round.iter().map(|game| game.home))
        .for_each(|(computed, parsed)| assert_eq!(computed, parsed));
}

#[test]
fn euro2020_id_check() {
    let (_, mut round, parsed_playoff) = euro_2020_setup();

    let mut parsed_first_round = parsed_playoff
        .games()
        .cloned()
        .collect::<Vec<Euro2020ParsePlayoffGame>>();

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
fn euro2020_team_to_game_assignment() {
    let (data, mut round, parsed_playoff) = euro_2020_setup();
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

fn fifa_2018_setup() -> (Fifa2018Data, Fifa2018Data, Round, Fifa2018arsePlayoff) {
    let blank_data: Fifa2018Data = get_data("lsv_data/blank-fifa-2018.json").unwrap();
    let complete_data: Fifa2018Data = get_data("lsv_data/complete-fifa-2018.json").unwrap();

    let groups = complete_data.try_groups().unwrap();
    let trans = blank_data.try_playoff_transitions().unwrap();

    let group_rules = fifa_2018_rules();
    let third_place_rules = noop_fifa_2018_third_place_rules();

    let computed_first_round =
        Round::first_round_from_group_stage(&groups, &trans, &group_rules, &third_place_rules);
    let parsed_first_round = blank_data.playoff.clone();
    (
        blank_data,
        complete_data,
        computed_first_round,
        parsed_first_round,
    )
}

fn euro_2020_setup() -> (Euro2020Data, Round, Euro2020ParsePlayoff) {
    let data: Euro2020Data = get_data("lsv_data/complete-euro-2020.json").unwrap();

    let groups = data.try_groups().unwrap();
    let teams = data.try_teams().unwrap();
    let ranking = UefaRanking::try_new(
        &groups,
        teams.iter().map(|(id, team)| (*id, team.rank)).collect(),
    )
    .expect("Failed to compile ranking");
    let trans = data.try_playoff_transitions().unwrap();

    let group_rules = euro_2020_rules(ranking.clone());
    let third_place_rules = euro_2020_third_place_rules(ranking);

    let computed_first_round =
        Round::first_round_from_group_stage(&groups, &trans, &group_rules, &third_place_rules);
    let parsed_first_round = data.playoff.clone();
    (data, computed_first_round, parsed_first_round)
}
