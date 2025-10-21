use itertools::Itertools;
use std::cmp::max;
use std::collections::HashMap;
use wwc_core::Team;
use wwc_core::game::GameId;
use wwc_core::group::order::{
    UefaRanking, euro_2020_rules, euro_2020_third_place_rules, fifa_2018_rules,
    noop_fifa_2018_third_place_rules,
};
use wwc_core::group::{Group, GroupOutcome};
use wwc_core::playoff::templates::single_elimination_8;
use wwc_core::playoff::transition::{PlayoffTransition, PlayoffTransitions};
use wwc_core::playoff::{
    BracketState, BracketStructure, BracketTemplate, PlayoffGameState, bracket,
};
use wwc_core::team::{FifaCode, TeamId};
use wwc_data::lsv::euro_2020::playoff::{
    ParsePlayoff as Euro2020ParsePlayoff, ParsePlayoffGame as Euro2020ParsePlayoffGame,
};
use wwc_data::lsv::fifa_2018::playoff::{
    ParsePlayoff as Fifa2018arsePlayoff, ParsePlayoffGame as Fifa2018arsePlayoffGame,
};
use wwc_data::lsv::{Euro2020Data, Fifa2018Data, LsvData, get_data};

// fn generate_bracket(start_id: GameId) {
//     for game_id in start_id..=GameId::from(100) {
//         let game = GameId::from(game_id);
//         // Implement logic to generate bracket for each game
//     }
// }

fn largest_group_game_id<'a>(groups: impl Iterator<Item = &'a Group>) -> GameId {
    let (played, unplayed) = groups.tee();
    let played = played
        .flat_map(|group| group.played_games())
        .map(|game| game.id)
        .max()
        .unwrap_or(GameId::from(0));
    let unplayed = unplayed
        .flat_map(|group| group.unplayed_games())
        .map(|game| game.id)
        .max()
        .unwrap_or(GameId::from(0));
    max(played, unplayed)
}

#[test]
fn euro_2020_playoff() {
    let blank_data: Euro2020Data = get_data("lsv_data/blank-euro-2020.json").unwrap();
    let complete_data: Euro2020Data = get_data("lsv_data/complete-euro-2020.json").unwrap();
    assert!(blank_data.playoff.games().count() == 15);

    let groups = complete_data.try_groups().unwrap();

    assert_eq!(largest_group_game_id(groups.values()), GameId::from(36));
    let teams = blank_data.try_teams().unwrap();
    let ranking = UefaRanking::try_new(
        &groups,
        teams.iter().map(|(id, team)| (*id, team.rank)).collect(),
    )
    .expect("Failed to compile ranking");
    let rules = euro_2020_rules(ranking);
    let bracket_struct = BracketStructure::from_team_sources(&blank_data.team_sources)
        .expect("Failed to create bracket structure");
    let bracket_state = BracketState::new();
    let computed_first_round = bracket_struct.games_at_depth(3, &bracket_state, &groups, &rules);
    let parsed_first_round = complete_data.playoff.games().take(8);

    // Ensure all games in the first round are Ready (teams are known)
    // And that the teams are correct
    computed_first_round
        .iter()
        .zip(parsed_first_round)
        .all(|(computed, parsed)| compare_games_equal(computed, parsed, &teams));

    // Play games
    let mut bracket_state = bracket_state;

    let code_to_id_map = teams
        .clone()
        .into_iter()
        .map(|(id, team)| (team.fifa_code, id))
        .collect();
    for game in complete_data.playoff.games() {
        let parsed_game = game.clone().try_parse(&code_to_id_map).unwrap();
        if let PlayoffGameState::Played { game_id, result } = parsed_game {
            bracket_state =
                bracket_state.play_game(game_id, result.home, result.away, result.score);
        }
    }

    // Check that computed winner is correct
    let computed_winner = teams
        .get(&bracket_struct.champion(&bracket_state).unwrap())
        .unwrap();
    assert_eq!(
        complete_data
            .playoff
            .games()
            .last()
            .unwrap()
            .winner
            .clone()
            .unwrap(),
        computed_winner.fifa_code
    );
}

fn compare_games_equal(
    computed: &PlayoffGameState,
    parsed: &Euro2020ParsePlayoffGame,
    teams_map: &HashMap<TeamId, Team>,
) -> bool {
    println!("{:?} \n {:?} \n\n", computed, parsed);
    match computed {
        PlayoffGameState::Pending {
            game_id,
            home_source,
            away_source,
        } => {
            let correct_game_id = game_id == &parsed.id;
            let teams_pending = parsed.home_team.is_none() && parsed.away_team.is_none();
            correct_game_id && teams_pending
        }
        PlayoffGameState::HomeKnown {
            game_id,
            home,
            away_source,
        } => todo!(),
        PlayoffGameState::AwayKnown {
            game_id,
            home_source,
            away,
        } => todo!(),
        PlayoffGameState::Ready {
            game_id,
            home,
            away,
        } => {
            let correct_game_id = game_id == &parsed.id;
            let parsed = parsed.clone();
            let correct_teams = teams_map.get(&home).unwrap().fifa_code
                == FifaCode::try_from(parsed.home_team.unwrap()).unwrap()
                && teams_map.get(&away).unwrap().fifa_code
                    == FifaCode::try_from(parsed.away_team.unwrap()).unwrap();
            correct_game_id && correct_teams
        }
        PlayoffGameState::Played { game_id, result } => todo!(),
    }
}

// assert!(false);

// fn euro_2020_setup() -> (Euro2020Data, Round, Euro2020ParsePlayoff) {
//     let data: Euro2020Data = get_data("lsv_data/complete-euro-2020.json").unwrap();
//
//     let groups = data.try_groups().unwrap();
//     let teams = data.try_teams().unwrap();
//     let ranking = UefaRanking::try_new(
//         &groups,
//         teams.iter().map(|(id, team)| (*id, team.rank)).collect(),
//     )
//     .expect("Failed to compile ranking");
//     let trans = data.try_playoff_transitions().unwrap();
//
//     let group_rules = euro_2020_rules(ranking.clone());
//     let third_place_rules = euro_2020_third_place_rules(ranking);
//
//     let computed_first_round =
//         Round::first_round_from_group_stage(&groups, &trans, &group_rules, &third_place_rules);
//     let parsed_first_round = data.playoff.clone();
//     (data, computed_first_round, parsed_first_round)
// }
// #[test]
//
// fn fifa2018_id_check() {
//     let (_, _, mut round, parsed_playoff) = fifa_2018_setup();
//
//     let mut parsed_first_round = parsed_playoff
//         .games()
//         .cloned()
//         .collect::<Vec<Fifa2018arsePlayoffGame>>();
//
//     // Sort by id to ensure the correct games are compared
//     round.games.sort_by_key(|game| game.id);
//     parsed_first_round.sort_by_key(|game| game.id);
//
//     // Id check
//     round
//         .games
//         .iter()
//         .map(|game| game.id)
//         .zip(parsed_first_round.iter().map(|game| game.id))
//         .for_each(|(computed, parsed)| assert_eq!(computed, parsed));
// }
//
// #[test]
// fn fifa2018_team_to_game_assignment() {
//     let (_, complete_data, mut round, _) = fifa_2018_setup();
//     let mut parsed_first_round = complete_data
//         .playoff
//         .games()
//         .map(|game| {
//             PlayoffGame::new(
//                 game.id,
//                 game.home_team.team_from_finished().unwrap(),
//                 game.away_team.team_from_finished().unwrap(),
//             )
//         })
//         .collect::<Vec<PlayoffGame>>();
//
//     // Sort by id to ensure the correct games are compared
//     parsed_first_round.sort_by_key(|game| game.id);
//     round.games.sort_by_key(|game| game.id);
//
//     parsed_first_round
//         .iter()
//         .zip(round.iter())
//         .for_each(|(parsed, comp)| {
//             println!(
//                 "Id: {}=?{}\n\thome:{}=?{}\n\taway:{}=?{}\n\n\n",
//                 parsed.id,
//                 comp.id,
//                 parsed.home.unwrap(),
//                 comp.home.unwrap(),
//                 parsed.away.unwrap(),
//                 comp.away.unwrap()
//             )
//         });
//     round
//         .games
//         .iter()
//         .map(|game| game.home)
//         .zip(parsed_first_round.iter().map(|game| game.home))
//         .for_each(|(computed, parsed)| assert_eq!(computed, parsed));
// }
//
// #[test]
// fn euro2020_id_check() {
//     let (_, mut round, parsed_playoff) = euro_2020_setup();
//
//     let mut parsed_first_round = parsed_playoff
//         .games()
//         .cloned()
//         .collect::<Vec<Euro2020ParsePlayoffGame>>();
//
//     // Sort by id to ensure the correct games are compared
//     round.games.sort_by_key(|game| game.id);
//     parsed_first_round.sort_by_key(|game| game.id);
//
//     // Id check
//     round
//         .games
//         .iter()
//         .map(|game| game.id)
//         .zip(parsed_first_round.iter().map(|game| game.id))
//         .for_each(|(computed, parsed)| assert_eq!(computed, parsed));
// }

// #[test]
// fn euro2020_team_to_game_assignment() {
//     let (data, mut round, parsed_playoff) = euro_2020_setup();
//     let team_map = data.team_map;
//     let mut parsed_first_round = parsed_playoff
//         .games()
//         .map(|game| {
//             PlayoffGame::new(
//                 game.id,
//                 *team_map.get(&game.home_team.clone().unwrap()).unwrap(),
//                 *team_map.get(&game.away_team.clone().unwrap()).unwrap(),
//             )
//         })
//         .collect::<Vec<PlayoffGame>>();
//
//     // Sort by id to ensure the correct games are compared
//     parsed_first_round.sort_by_key(|game| game.id);
//     round.games.sort_by_key(|game| game.id);
//
//     round
//         .games
//         .iter()
//         .map(|game| game.home)
//         .zip(parsed_first_round.iter().map(|game| game.home))
//         .for_each(|(computed, parsed)| assert_eq!(computed, parsed));
// }
//
// fn fifa_2018_setup() -> (Fifa2018Data, Fifa2018Data, Round, Fifa2018arsePlayoff) {
//     let blank_data: Fifa2018Data = get_data("lsv_data/blank-fifa-2018.json").unwrap();
//     let complete_data: Fifa2018Data = get_data("lsv_data/complete-fifa-2018.json").unwrap();
//
//     let groups = complete_data.try_groups().unwrap();
//     let trans = blank_data.try_playoff_transitions().unwrap();
//
//     let group_rules = fifa_2018_rules();
//     let third_place_rules = noop_fifa_2018_third_place_rules();
//
//     let computed_first_round =
//         Round::first_round_from_group_stage(&groups, &trans, &group_rules, &third_place_rules);
//     let parsed_first_round = blank_data.playoff.clone();
//     (
//         blank_data,
//         complete_data,
//         computed_first_round,
//         parsed_first_round,
//     )
// }
