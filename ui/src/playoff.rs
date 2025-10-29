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
pub fn PlayoffBracketView<T, F1, F2>(
    bracket: BracketStructure,
    bracket_state: BracketState,
    groups: Groups,
    teams: HashMap<TeamId, Team>,
    rules: Rules<T>,
    on_play: F1,
    on_unplay: F2,
) -> impl IntoView
where
    T: Tiebreaker + 'static,
    F1: Fn(PlayoffScoreInput) + Clone + Send + Sync + 'static,
    F2: Fn(GameId) + Clone + Send + Sync + 'static,
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
                        view! {
                            <ul class="tournament-bracket__round">
                                <ul class="tournament-bracket__list">
                                    {games
                                        .into_iter()
                                        .map(|game| {
                                            view_playoff_game(
                                                game,
                                                teams.clone(),
                                                on_play.clone(),
                                                on_unplay.clone(),
                                            )
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

fn view_playoff_game<F1, F2>(
    game: PlayoffGameState,
    teams: HashMap<TeamId, Team>,
    on_play: F1,
    on_unplay: F2,
) -> impl IntoView
where
    F1: Fn(PlayoffScoreInput) + Clone + Send + Sync + 'static,
    F2: Fn(GameId) + Clone + Send + Sync + 'static,
{
    match game {
        PlayoffGameState::Pending {
            game_id,
            home_source,
            away_source,
        } => view! {
            <PendingGameView game_id=game_id home_source=home_source away_source=away_source/>
        }
        .into_any(),
        PlayoffGameState::HomeKnown {
            game_id,
            home,
            away_source,
        } => view! {
            <HomeKnownGameView game_id=game_id home=home away_source=away_source teams=teams/>
        }
        .into_any(),
        PlayoffGameState::AwayKnown {
            game_id,
            home_source,
            away,
        } => view! {
            <AwayKnownGameView
                game_id=game_id
                home_source=home_source
                away=away
                teams=teams
            />
        }
        .into_any(),
        PlayoffGameState::Ready {
            game_id,
            home,
            away,
        } => view! {
            <ReadyGameView game_id=game_id home=home away=away teams=teams on_play=on_play/>
        }
        .into_any(),
        PlayoffGameState::Played { game_id, result } => view! {
            <PlayedGameView game_id=game_id result=result teams=teams on_unplay=on_unplay/>
        }
        .into_any(),
    }
}
