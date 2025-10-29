use crate::team::format_team_flag;
use leptos::ev;
use leptos::prelude::*;
use wwc_core::game::{GameId, GoalCount};
use wwc_core::playoff::{PlayoffResult, PlayoffScore, TeamSource};
use wwc_core::team::{TeamId, Teams};

/// Input data for playing a playoff game
#[derive(Debug, Clone, Copy)]
pub struct PlayoffScoreInput {
    pub game_id: GameId,
    pub home: TeamId,
    pub away: TeamId,
    pub score: PlayoffScore,
}

impl PlayoffScoreInput {
    pub fn new(game_id: GameId, home: TeamId, away: TeamId, score: PlayoffScore) -> Self {
        Self {
            game_id,
            home,
            away,
            score,
        }
    }
}

/// Shared helper: Format team source as text
fn format_source_display(source: &TeamSource) -> String {
    source.to_string()
}

/// Shared helper: Wrap content in game box structure
fn game_box_wrapper(content: impl IntoView, class: &str) -> impl IntoView {
    view! {
        <li class="tournament-bracket__item">
            <div class={format!("tournament-bracket__match {}", class)}>
                {content}
            </div>
        </li>
    }
}

/// Component for Pending state - both teams unknown
#[component]
pub fn PendingGameView(
    game_id: GameId,
    home_source: TeamSource,
    away_source: TeamSource,
) -> impl IntoView {
    let home_text = format_source_display(&home_source);
    let away_text = format_source_display(&away_source);

    let content = view! {
        <div class="playoff-team-placeholder">
            <span class="tournament-bracket__source">{home_text}</span>
        </div>
        <span class="vs-separator">"-"</span>
        <div class="playoff-team-placeholder">
            <span class="tournament-bracket__source">{away_text}</span>
        </div>
    };

    game_box_wrapper(content, "playoff-pending")
}

/// Component for HomeKnown state - only home team known
#[component]
pub fn HomeKnownGameView(
    game_id: GameId,
    home: TeamId,
    away_source: TeamSource,
    teams: Teams,
) -> impl IntoView {
    let home_team = teams.get(&home).cloned();
    let away_text = format_source_display(&away_source);

    let content = view! {
        <div class="playoff-team-known">
            {if let Some(team) = home_team {
                let flag = format_team_flag(&team);
                let code = team.fifa_code.to_string();
                view! {
                    <span class={flag}></span>
                    <span class="tournament-bracket__code">{code}</span>
                }
                    .into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
        </div>
        <span class="vs-separator">"-"</span>
        <div class="playoff-team-placeholder">
            <span class="tournament-bracket__source">{away_text}</span>
        </div>
    };

    game_box_wrapper(content, "playoff-partial")
}

/// Component for AwayKnown state - only away team known
#[component]
pub fn AwayKnownGameView(
    game_id: GameId,
    home_source: TeamSource,
    away: TeamId,
    teams: Teams,
) -> impl IntoView {
    let home_text = format_source_display(&home_source);
    let away_team = teams.get(&away).cloned();

    let content = view! {
        <div class="playoff-team-placeholder">
            <span class="tournament-bracket__source">{home_text}</span>
        </div>
        <span class="vs-separator">"-"</span>
        <div class="playoff-team-known">
            {if let Some(team) = away_team {
                let flag = format_team_flag(&team);
                let code = team.fifa_code.to_string();
                view! {
                    <span class={flag}></span>
                    <span class="tournament-bracket__code">{code}</span>
                }
                    .into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
        </div>
    };

    game_box_wrapper(content, "playoff-partial")
}

/// Component for Ready state - both teams known, can be predicted
#[component]
pub fn ReadyGameView(
    game_id: GameId,
    home: TeamId,
    away: TeamId,
    teams: Teams,
    on_play: impl Fn(PlayoffScoreInput) + Clone + 'static,
) -> impl IntoView {
    let home_team = teams.get(&home).cloned();
    let away_team = teams.get(&away).cloned();

    let home_input_ref = NodeRef::<leptos::html::Input>::new();
    let away_input_ref = NodeRef::<leptos::html::Input>::new();
    let home_penalty_ref = NodeRef::<leptos::html::Input>::new();
    let away_penalty_ref = NodeRef::<leptos::html::Input>::new();

    let (is_draw, set_is_draw) = signal(false);

    let check_draw = move || {
        if let (Some(home_input), Some(away_input)) = (home_input_ref.get(), away_input_ref.get()) {
            let home_val = home_input.value();
            let away_val = away_input.value();

            if let (Ok(home_score), Ok(away_score)) =
                (home_val.parse::<u32>(), away_val.parse::<u32>())
            {
                set_is_draw.set(home_score == away_score);
            } else {
                set_is_draw.set(false);
            }
        }
    };

    let on_home_keydown = move |ev: ev::KeyboardEvent| {
        if ev.key() == "Tab" || ev.key() == "Enter" {
            check_draw();
            if let Some(away_input) = away_input_ref.get() {
                let _ = away_input.focus();
                ev.prevent_default();
            }
        }
    };

    let on_play_clone = on_play.clone();
    let on_away_keydown = move |ev: ev::KeyboardEvent| {
        check_draw();
        if ev.key() == "Tab" || ev.key() == "Enter" {
            if is_draw.get_untracked() {
                if let Some(home_pen_input) = home_penalty_ref.get() {
                    let _ = home_pen_input.focus();
                    ev.prevent_default();
                }
            } else {
                // Not a draw, submit immediately
                if ev.key() == "Enter" {
                    if let (Some(home_input), Some(away_input)) =
                        (home_input_ref.get(), away_input_ref.get())
                    {
                        let home_val = home_input.value();
                        let away_val = away_input.value();

                        if let (Ok(home_score), Ok(away_score)) =
                            (home_val.parse::<u32>(), away_val.parse::<u32>())
                        {
                            if let (Ok(home_goals), Ok(away_goals)) = (
                                GoalCount::try_from(home_score),
                                GoalCount::try_from(away_score),
                            ) {
                                if let Ok(score) =
                                    PlayoffScore::regular_time(home_goals, away_goals)
                                {
                                    on_play_clone(PlayoffScoreInput::new(
                                        game_id, home, away, score,
                                    ));
                                    home_input.set_value("");
                                    away_input.set_value("");
                                    set_is_draw.set(false);
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    let on_home_penalty_keydown = move |ev: ev::KeyboardEvent| {
        if ev.key() == "Tab" || ev.key() == "Enter" {
            if let Some(away_pen_input) = away_penalty_ref.get() {
                let _ = away_pen_input.focus();
                ev.prevent_default();
            }
        }
    };

    let on_away_penalty_keydown = move |ev: ev::KeyboardEvent| {
        if ev.key() == "Enter" {
            // Submit with penalties
            if let (Some(home_input), Some(away_input)) =
                (home_input_ref.get(), away_input_ref.get())
            {
                let home_val = home_input.value();
                let away_val = away_input.value();

                if let (Ok(home_score), Ok(away_score)) =
                    (home_val.parse::<u32>(), away_val.parse::<u32>())
                {
                    if let (Ok(home_goals), Ok(away_goals)) = (
                        GoalCount::try_from(home_score),
                        GoalCount::try_from(away_score),
                    ) {
                        if let (Some(home_pen_input), Some(away_pen_input)) =
                            (home_penalty_ref.get(), away_penalty_ref.get())
                        {
                            let home_pen_val = home_pen_input.value();
                            let away_pen_val = away_pen_input.value();

                            if let (Ok(home_pen), Ok(away_pen)) =
                                (home_pen_val.parse::<u32>(), away_pen_val.parse::<u32>())
                            {
                                if let (Ok(home_pen_goals), Ok(away_pen_goals)) =
                                    (GoalCount::try_from(home_pen), GoalCount::try_from(away_pen))
                                {
                                    if let Ok(score) = PlayoffScore::with_penalties(
                                        home_goals,
                                        away_goals,
                                        home_pen_goals,
                                        away_pen_goals,
                                    ) {
                                        on_play(PlayoffScoreInput::new(game_id, home, away, score));
                                        home_input.set_value("");
                                        away_input.set_value("");
                                        home_pen_input.set_value("");
                                        away_pen_input.set_value("");
                                        set_is_draw.set(false);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    let content = view! {
        <div class="playoff-ready-game">
            <div class="playoff-teams-row">
                <div class="playoff-team">
                    {if let Some(team) = home_team {
                        let flag = format_team_flag(&team);
                        let code = team.fifa_code.to_string();
                        view! {
                            <span class={flag}></span>
                            <span class="tournament-bracket__code">{code}</span>
                        }
                            .into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>
                <div class="playoff-score-inputs">
                    <input
                        node_ref=home_input_ref
                        class="playoff-score-input"
                        type="number"
                        min="0"
                        size=1
                        on:keydown=on_home_keydown
                        on:input=move |_| check_draw()
                    />
                    <span class="score-separator">"-"</span>
                    <input
                        node_ref=away_input_ref
                        class="playoff-score-input"
                        type="number"
                        min="0"
                        size=1
                        on:keydown=on_away_keydown
                        on:input=move |_| check_draw()
                    />
                </div>
                <div class="playoff-team">
                    {if let Some(team) = away_team {
                        let flag = format_team_flag(&team);
                        let code = team.fifa_code.to_string();
                        view! {
                            <span class={flag}></span>
                            <span class="tournament-bracket__code">{code}</span>
                        }
                            .into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>
            </div>
            <div
                class="playoff-penalty-row"
                style:display=move || if is_draw.get() { "flex" } else { "none" }
            >
                <span class="penalty-label">"Penalties:"</span>
                <input
                    node_ref=home_penalty_ref
                    class="playoff-score-input penalty-input"
                    type="number"
                    min="0"
                    size=1
                    on:keydown=on_home_penalty_keydown
                />
                <span class="score-separator">"-"</span>
                <input
                    node_ref=away_penalty_ref
                    class="playoff-score-input penalty-input"
                    type="number"
                    min="0"
                    size=1
                    on:keydown=on_away_penalty_keydown
                />
            </div>
        </div>
    };

    game_box_wrapper(content, "playoff-ready")
}

/// Component for Played state - game completed
#[component]
pub fn PlayedGameView(
    game_id: GameId,
    result: PlayoffResult,
    teams: Teams,
    on_unplay: impl Fn(GameId) + 'static,
) -> impl IntoView {
    let home_team = teams.get(&result.home).cloned();
    let away_team = teams.get(&result.away).cloned();

    let (home_score, away_score) = result.score.regular_time_score();
    let penalty_score = result.score.penalty_score();

    let score_display = if let Some((home_pen, away_pen)) = penalty_score {
        format!(
            "{}-{} ({}-{} pen)",
            home_score, away_score, home_pen, away_pen
        )
    } else {
        format!("{}-{}", home_score, away_score)
    };

    let content = view! {
        <div class="playoff-played-game">
            <div class="playoff-team">
                {if let Some(team) = home_team {
                    let flag = format_team_flag(&team);
                    let code = team.fifa_code.to_string();
                    view! {
                        <span class={flag}></span>
                        <span class="tournament-bracket__code">{code}</span>
                    }
                        .into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}
            </div>
            <div class="playoff-score-display">
                <span class="score-text">{score_display}</span>
                <button class="unplay-button" on:click=move |_| on_unplay(game_id)>
                    "\u{1F504}"
                </button>
            </div>
            <div class="playoff-team">
                {if let Some(team) = away_team {
                    let flag = format_team_flag(&team);
                    let code = team.fifa_code.to_string();
                    view! {
                        <span class={flag}></span>
                        <span class="tournament-bracket__code">{code}</span>
                    }
                        .into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}
            </div>
        </div>
    };

    game_box_wrapper(content, "playoff-played")
}
