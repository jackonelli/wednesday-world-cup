use crate::group_game::{PlayedGameView, ScoreInput, UnplayedGameView};
use crate::table::DisplayTableView;
use leptos::prelude::*;
use wwc_core::{
    game::GameId,
    group::{
        Group, GroupId, Groups,
        order::{fifa_2018_rules, order_group},
    },
    team::Teams,
};

pub(crate) fn view_group_play<U, V>(
    groups: Groups,
    teams: Teams,
    on_play: U,
    on_unplay: V,
) -> impl IntoView + use<U, V>
where
    U: Fn(ScoreInput) + Clone + 'static,
    V: Fn(GroupId, GameId) + Clone + 'static,
{
    let groups_vec: Vec<_> = groups.iter().map(|(id, g)| (*id, g.clone())).collect();

    view! {
        <section class="group_play">
            <div class="groups-container">
                {groups_vec
                    .into_iter()
                    .map(move |(group_id, group)| {
                        view_group(
                            group_id,
                            group,
                            teams.clone(),
                            on_play.clone(),
                            on_unplay.clone(),
                        )
                    })
                    .collect_view()}
            </div>
        </section>
    }
}

fn view_group<U, V>(
    id: GroupId,
    group: Group,
    teams: Teams,
    on_play: U,
    on_unplay: V,
) -> impl IntoView
where
    U: Fn(ScoreInput) + Clone + 'static,
    V: Fn(GroupId, GameId) + Clone + 'static,
{
    let rules = fifa_2018_rules();
    let group_order = order_group(&group, &rules);

    view! {
        <div class="group">
            <h3>{id.to_string()}</h3>
            <DisplayTableView
                group=group.clone()
                teams=teams.clone()
                group_order=group_order
            />
            {format_group_games(id, group, teams, on_play, on_unplay)}
        </div>
    }
}

fn format_group_games<U, V>(
    group_id: GroupId,
    group: Group,
    teams: Teams,
    on_play: U,
    on_unplay: V,
) -> impl IntoView
where
    U: Fn(ScoreInput) + Clone + 'static,
    V: Fn(GroupId, GameId) + Clone + 'static,
{
    let played_games: Vec<_> = group.played_games().collect();
    let unplayed_games: Vec<_> = group.unplayed_games().collect();

    view! {
        <div class="games">
            <table>
                {played_games
                    .into_iter()
                    .map(|game| {
                        view! {
                            <PlayedGameView
                                game=*game
                                teams=teams.clone()
                                group_id=group_id
                                on_unplay=on_unplay.clone()
                            />
                        }
                    })
                    .collect_view()}
                {unplayed_games
                    .into_iter()
                    .map(|game| {
                        view! {
                            <UnplayedGameView
                                game=*game
                                teams=teams.clone()
                                group_id=group_id
                                on_play=on_play.clone()
                            />
                        }
                    })
                    .collect_view()}
            </table>
        </div>
    }
}
