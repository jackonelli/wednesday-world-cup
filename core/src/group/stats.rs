use crate::fair_play::FairPlayValue;
use crate::game::{GoalCount, GoalDiff};
use crate::group::game::PlayedGroupGame;
use crate::group::{Group, GroupPoint};
use crate::team::{Rank, TeamId};
use std::collections::{HashMap, HashSet};
use std::ops;

pub trait UnaryStat: Ord + Copy + num::Zero + ops::AddAssign {
    fn stat(game: &PlayedGroupGame) -> (Self, Self);

    fn team_stats(group: &Group) -> HashMap<TeamId, Self> {
        let team_map = group.teams().map(|team| (team, Self::zero())).collect();
        group.played_games.iter().fold(team_map, |mut acc, game| {
            let (delta_home_stat, delta_away_stat) = Self::stat(game);

            let stats = acc
                .get_mut(&game.home)
                // TeamId will always be present, checked in Group constructor
                .unwrap();
            *stats += delta_home_stat;

            let stats = acc
                .get_mut(&game.away)
                // TeamId will always be present, checked in Group constructor
                .unwrap();
            *stats += delta_away_stat;
            acc
        })
    }

    fn internal_team_stats(group: &Group, team_filter: &HashSet<TeamId>) -> HashMap<TeamId, Self> {
        let team_map = team_filter
            .iter()
            .map(|team| (*team, Self::zero()))
            .collect();
        group
            .played_games
            .iter()
            .filter(|game| team_filter.contains(&game.home) && team_filter.contains(&game.away))
            .fold(team_map, |mut acc, game| {
                let (delta_home_stat, delta_away_stat) = Self::stat(game);

                let stats = acc
                    .get_mut(&game.home)
                    // TeamId will always be present, checked in Group constructor
                    .unwrap();
                *stats += delta_home_stat;

                let stats = acc
                    .get_mut(&game.away)
                    // TeamId will always be present, checked in Group constructor
                    .unwrap();
                *stats += delta_away_stat;
                acc
            })
    }
}

impl UnaryStat for GroupPoint {
    fn stat(game: &PlayedGroupGame) -> (Self, Self) {
        let score = &game.score;
        if score.home > score.away {
            (GroupPoint(3), GroupPoint(0))
        } else if score.home < score.away {
            (GroupPoint(0), GroupPoint(3))
        } else {
            (GroupPoint(1), GroupPoint(1))
        }
    }
}

impl UnaryStat for GoalDiff {
    fn stat(game: &PlayedGroupGame) -> (Self, Self) {
        let goal_diff = game.score.home - game.score.away;
        (goal_diff, -goal_diff)
    }
}

impl UnaryStat for GoalCount {
    fn stat(game: &PlayedGroupGame) -> (Self, Self) {
        (game.score.home, game.score.away)
    }
}

impl UnaryStat for FairPlayValue {
    fn stat(game: &PlayedGroupGame) -> (Self, Self) {
        (game.fair_play.home, game.fair_play.away)
    }
}

#[cfg(test)]
mod tests {}
