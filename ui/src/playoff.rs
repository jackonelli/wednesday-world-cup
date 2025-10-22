use leptos::prelude::*;
use web_sys::console;
use wwc_core::group::Groups;
use wwc_core::group::order::{Rules, Tiebreaker};
use wwc_core::playoff::{BracketState, BracketStructure, PlayoffGameState};

#[component]
pub fn PlayoffBracketView<T>(
    bracket: BracketStructure,
    groups: Groups,
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
            <h2>"Playoff Bracket"</h2>
            <div class="tournament-bracket tournament-bracket--rounded">
                {rounds
                    .into_iter()
                    .map(|depth| {
                        let round_name = round_name(depth, max_depth);
                        let games = bracket.games_at_depth(depth, &state, &groups, &rules);
                        view! {
                            <ul class="tournament-bracket__round">
                                <li class="tournament-bracket__round-title">{round_name}</li>
                                <ul class="tournament-bracket__list">
                                    {games
                                        .into_iter()
                                        .map(|game| view_playoff_game(game))
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

fn round_name(depth: usize, max_depth: usize) -> &'static str {
    match depth {
        0 => "Final",
        1 if max_depth == 1 => "Semi-Finals",
        1 => "Semi-Finals",
        2 => "Quarter-Finals",
        3 => "Round of 16",
        4 => "Round of 32",
        _ => "Round",
    }
}

fn view_playoff_game(game: PlayoffGameState) -> impl IntoView {
    match game {
        PlayoffGameState::Pending {
            game_id,
            home_source,
            away_source,
        } => {
            view! {
                <li class="tournament-bracket__item">
                    <div class="tournament-bracket__match">
                        <table class="tournament-bracket__table">
                            <tbody class="tournament-bracket__content">
                                <tr class="tournament-bracket__team">
                                    <td class="tournament-bracket__country">
                                        <span class="tournament-bracket__code">{home_source.to_string()}</span>
                                    </td>
                                </tr>
                                <tr class="tournament-bracket__team">
                                    <td class="tournament-bracket__country">
                                        <span class="tournament-bracket__code">{away_source.to_string()}</span>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </li>
            }
        }
        PlayoffGameState::HomeKnown {
            game_id,
            home,
            away_source,
        } => {
            view! {
                <li class="tournament-bracket__item">
                    <div class="tournament-bracket__match">
                        <table class="tournament-bracket__table">
                            <tbody class="tournament-bracket__content">
                                <tr class="tournament-bracket__team">
                                    <td class="tournament-bracket__country">
                                        <span class="tournament-bracket__code">{format!("Team {}", home)}</span>
                                    </td>
                                </tr>
                                <tr class="tournament-bracket__team">
                                    <td class="tournament-bracket__country">
                                        <span class="tournament-bracket__code">{away_source.to_string()}</span>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </li>
            }
        }
        PlayoffGameState::AwayKnown {
            game_id,
            home_source,
            away,
        } => {
            view! {
                <li class="tournament-bracket__item">
                    <div class="tournament-bracket__match">
                        <table class="tournament-bracket__table">
                            <tbody class="tournament-bracket__content">
                                <tr class="tournament-bracket__team">
                                    <td class="tournament-bracket__country">
                                        <span class="tournament-bracket__code">{home_source.to_string()}</span>
                                    </td>
                                </tr>
                                <tr class="tournament-bracket__team">
                                    <td class="tournament-bracket__country">
                                        <span class="tournament-bracket__code">{format!("Team {}", away)}</span>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </li>
            }
        }
        PlayoffGameState::Ready {
            game_id,
            home,
            away,
        } => {
            view! {
                <li class="tournament-bracket__item">
                    <div class="tournament-bracket__match">
                        <table class="tournament-bracket__table">
                            <tbody class="tournament-bracket__content">
                                <tr class="tournament-bracket__team">
                                    <td class="tournament-bracket__country">
                                        <span class="tournament-bracket__code">{format!("Team {}", home)}</span>
                                    </td>
                                </tr>
                                <tr class="tournament-bracket__team">
                                    <td class="tournament-bracket__country">
                                        <span class="tournament-bracket__code">{format!("Team {}", away)}</span>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </li>
            }
        }
        PlayoffGameState::Played { game_id, result } => {
            view! {
                <li class="tournament-bracket__item">
                    <div class="tournament-bracket__match">
                        <table class="tournament-bracket__table">
                            <tbody class="tournament-bracket__content">
                                <tr class="tournament-bracket__team tournament-bracket__team--winner">
                                    <td class="tournament-bracket__country">
                                        <span class="tournament-bracket__code">{format!("Team {}", result.home)}</span>
                                    </td>
                                </tr>
                                <tr class="tournament-bracket__team">
                                    <td class="tournament-bracket__country">
                                        <span class="tournament-bracket__code">{format!("Team {}", result.away)}</span>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </li>
            }
        }
    }
}
