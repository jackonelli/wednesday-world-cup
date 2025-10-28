use leptos::prelude::*;
use std::collections::HashMap;
use wwc_core::group::Groups;
use wwc_core::group::order::{Rules, Tiebreaker};
use wwc_core::playoff::{BracketState, BracketStructure, PlayoffGameState};
use wwc_core::team::{Team, TeamId};

#[component]
pub fn PlayoffBracketView<T>(
    bracket: BracketStructure,
    groups: Groups,
    teams: HashMap<TeamId, Team>,
    rules: Rules<T>,
) -> impl IntoView
where
    T: Tiebreaker + 'static,
{
    let max_depth = bracket.max_depth();
    let state = BracketState::new(); // Empty state - no games played yet
    let rounds: Vec<_> = (0..=max_depth).rev().collect();

    view! {
        <section class="playoff-bracket">
            <div class="tournament-bracket tournament-bracket--rounded">
                {rounds
                    .into_iter()
                    .map(|depth| {
                        let games = bracket.games_at_depth(depth, &state, &groups, &rules);
                        view! {
                            <ul class="tournament-bracket__round">
                                <ul class="tournament-bracket__list">
                                    {games
                                        .into_iter()
                                        .map(|game| view_playoff_game(game, teams.clone()))
                                        .collect_view()}
                                </ul>
                            </ul>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}

fn view_playoff_game(game: PlayoffGameState, teams: HashMap<TeamId, Team>) -> impl IntoView {
    match game {
        PlayoffGameState::Pending {
            game_id,
            home_source,
            away_source,
        } => view_game_box(home_source.to_string(), away_source.to_string()),
        PlayoffGameState::HomeKnown {
            game_id,
            home,
            away_source,
        } => {
            let home_text = get_team_text(&teams, &home);
            view_game_box(home_text, away_source.to_string())
        }
        PlayoffGameState::AwayKnown {
            game_id,
            home_source,
            away,
        } => {
            let away_text = get_team_text(&teams, &away);
            view_game_box(home_source.to_string(), away_text)
        }
        PlayoffGameState::Ready {
            game_id,
            home,
            away,
        } => {
            let home_text = get_team_text(&teams, &home);
            let away_text = get_team_text(&teams, &away);
            view_game_box(home_text, away_text)
        }
        PlayoffGameState::Played { game_id, result } => {
            let home_text = get_team_text(&teams, &result.home);
            let away_text = get_team_text(&teams, &result.away);
            view_game_box(home_text, away_text)
        }
    }
}

fn get_team_text(teams: &HashMap<TeamId, Team>, team_id: &TeamId) -> String {
    teams
        .get(team_id)
        .map(|t| t.fifa_code.to_string())
        .unwrap_or_else(|| format!("Team {}", team_id))
}

fn view_game_box(home_text: String, away_text: String) -> impl IntoView {
    view! {
        <li class="tournament-bracket__item">
            <div class="tournament-bracket__match">
                <span class="tournament-bracket__code">{home_text}</span>
                <span class="vs-separator">"-"</span>
                <span class="tournament-bracket__code">{away_text}</span>
            </div>
        </li>
    }
}
