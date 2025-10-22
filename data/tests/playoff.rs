use itertools::Itertools;
use std::cmp::max;
use std::collections::HashMap;
use wwc_core::Team;
use wwc_core::game::GameId;
use wwc_core::group::Group;
use wwc_core::group::order::{UefaRanking, euro_2020_rules, fifa_2018_rules};
use wwc_core::playoff::{BracketState, BracketStructure, PlayoffGameState};
use wwc_core::team::{FifaCode, TeamId};
use wwc_data::lsv::euro_2020::playoff::ParsePlayoffGame as Euro2020ParsePlayoffGame;
use wwc_data::lsv::fifa_2018::playoff::ParsePlayoffGame as Fifa2018arsePlayoffGame;
use wwc_data::lsv::{Euro2020Data, Fifa2018Data, LsvData, get_data};

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

#[test]
fn fifa_2018_playoff() {
    let blank_data: Fifa2018Data = get_data("lsv_data/blank-fifa-2018.json").unwrap();
    let complete_data: Fifa2018Data = get_data("lsv_data/complete-fifa-2018.json").unwrap();
    assert_eq!(blank_data.playoff.games().count(), 16); // 8 + 4 + 2 + 1 + 1 (3rd place)

    let groups = complete_data.try_groups().unwrap();

    assert_eq!(largest_group_game_id(groups.values()), GameId::from(48));
    let teams = blank_data.try_teams().unwrap();
    let rules = fifa_2018_rules();

    // Team sources are now parsed from all playoff rounds in the JSON
    let bracket_struct = BracketStructure::from_team_sources(&blank_data.team_sources)
        .expect("Failed to create bracket structure");
    let bracket_state = BracketState::new();
    // For single elimination 16-team bracket: depth 0 = final, 1 = semis, 2 = quarters, 3 = round of 16
    let mut computed_first_round =
        bracket_struct.games_at_depth(3, &bracket_state, &groups, &rules);
    let mut parsed_first_round = complete_data.playoff.round_16.games.clone();

    // Sort both by game ID to ensure correct comparison
    computed_first_round.sort_by_key(|g| g.game_id());
    parsed_first_round.sort_by_key(|g| g.id);

    // Ensure all games in the first round are Ready (teams are known)
    // And that the teams are correct
    computed_first_round
        .iter()
        .zip(parsed_first_round.iter())
        .all(|(computed, parsed)| compare_fifa_games_equal(computed, parsed, &teams));

    // Play games
    let mut bracket_state = bracket_state;

    let id_to_id_map: HashMap<TeamId, TeamId> = teams
        .clone()
        .into_iter()
        .map(|(id, _team)| (id, id))
        .collect();
    for game in complete_data.playoff.games() {
        let parsed_game = game.clone().try_parse(&id_to_id_map).unwrap();
        if let PlayoffGameState::Played { game_id, result } = parsed_game {
            bracket_state =
                bracket_state.play_game(game_id, result.home, result.away, result.score);
        }
    }

    // Check that computed winner is correct
    let computed_winner_id = bracket_struct.champion(&bracket_state).unwrap();
    // The final game is in round_2, not the last game (which is the 3rd place playoff)
    let expected_winner_id = complete_data
        .playoff
        .round_2
        .games
        .first()
        .unwrap()
        .winner
        .unwrap();
    assert_eq!(computed_winner_id, expected_winner_id);
}

fn compare_fifa_games_equal(
    computed: &PlayoffGameState,
    parsed: &Fifa2018arsePlayoffGame,
    teams_map: &HashMap<TeamId, Team>,
) -> bool {
    use wwc_data::lsv::fifa_2018::playoff::ParsePlayoffTransition;

    match computed {
        PlayoffGameState::Pending {
            game_id,
            home_source,
            away_source,
        } => {
            let correct_game_id = game_id == &parsed.id;
            let teams_pending = matches!(parsed.home_team, ParsePlayoffTransition::UnFinished(_))
                && matches!(parsed.away_team, ParsePlayoffTransition::UnFinished(_));
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
            let parsed_home_id = match &parsed.home_team {
                ParsePlayoffTransition::Finished(id) => Some(*id),
                _ => None,
            };
            let parsed_away_id = match &parsed.away_team {
                ParsePlayoffTransition::Finished(id) => Some(*id),
                _ => None,
            };
            let correct_teams = parsed_home_id == Some(*home) && parsed_away_id == Some(*away);
            correct_game_id && correct_teams
        }
        PlayoffGameState::Played { game_id, result } => todo!(),
    }
}
