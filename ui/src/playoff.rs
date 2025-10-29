use crate::playoff_game::{
    AwayKnownGameView, HomeKnownGameView, PendingGameView, PlayedGameView, PlayoffScoreInput,
    ReadyGameView,
};
use leptos::prelude::*;
use std::collections::HashMap;
use wwc_core::game::GameId;
use wwc_core::group::Groups;
use wwc_core::group::order::{Rules, Tiebreaker};
use wwc_core::playoff::{BracketState, BracketStructure, PlayoffGameState};
use wwc_core::team::{Team, TeamId};

#[component]
pub fn PlayoffBracketView<T>(
    bracket: BracketStructure,
    bracket_state: BracketState,
    groups: Groups,
    teams: HashMap<TeamId, Team>,
    rules: Rules<T>,
    on_play: impl Fn(PlayoffScoreInput) + Clone + 'static,
    on_unplay: impl Fn(GameId) + Clone + 'static,
) -> impl IntoView
where
    T: Tiebreaker + 'static,
{
    let max_depth = bracket.max_depth();
    let rounds: Vec<_> = (0..=max_depth).rev().collect();

    view! {
        <section class="playoff-bracket">
            <div class="tournament-bracket tournament-bracket--rounded">
                {rounds
                    .into_iter()
                    .map(|depth| {
                        let games = bracket.games_at_depth(depth, &bracket_state, &groups, &rules);
                        let on_play = on_play.clone();
                        let on_unplay = on_unplay.clone();
                        let teams = teams.clone();
                        view! {
                            <ul class="tournament-bracket__round">
                                <ul class="tournament-bracket__list">
                                    {games
                                        .into_iter()
                                        .map(|game| {
                                            let teams = teams.clone();
                                            let on_play = on_play.clone();
                                            let on_unplay = on_unplay.clone();

                                            match game {
                                                PlayoffGameState::Pending { game_id, home_source, away_source } =>
                                                    view! { <PendingGameView game_id home_source away_source/> }.into_any(),
                                                PlayoffGameState::HomeKnown { game_id, home, away_source } =>
                                                    view! { <HomeKnownGameView game_id home away_source teams/> }.into_any(),
                                                PlayoffGameState::AwayKnown { game_id, home_source, away } =>
                                                    view! { <AwayKnownGameView game_id home_source away teams/> }.into_any(),
                                                PlayoffGameState::Ready { game_id, home, away } =>
                                                    view! { <ReadyGameView game_id home away teams on_play/> }.into_any(),
                                                PlayoffGameState::Played { game_id, result } =>
                                                    view! { <PlayedGameView game_id result teams on_unplay/> }.into_any(),
                                            }
                                        })
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
