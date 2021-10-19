//! Tournament playoff
pub mod game;
pub mod transition;
use crate::group::{GroupOutcome, Groups};
use crate::playoff::game::PlayoffGame;
use crate::playoff::transition::{PlayoffTransition, PlayoffTransitions};
use crate::team::TeamId;
use crate::group::order::{Rules, Tiebreaker};
use thiserror::Error;

use crate::game::GameId;
use std::collections::HashMap;

struct Playoff {
    rounds: HashMap<RoundIdx, Round>,
}

struct Round {
    games: HashMap<GameId, PlayoffGame>,
}

impl Round {
    pub fn first_round_from_group_stage<T: Tiebreaker>(
        groups: &Groups,
        transitions: &PlayoffTransitions,
        rules: &Rules<T>,
    ) -> Self {
        transitions.iter().map(|(id, trans)| {})
    }
}

fn teams_from_groups<T: Tiebreaker>(groups: &Groups, trans: &PlayoffTransition, rules: &Rules<T>) -> (TeamId, TeamId) {
    let home = match trans.home {
        GroupOutcome::Winner(id) => groups.get(&id).unwrap().winner(rules),
        GroupOutcome::RunnerUp(id) => groups.get(&id).unwrap().runner_up(rules),
        GroupOutcome::ThirdPlace(ids) => todo!(),
    };
    let away = match trans.away {
        GroupOutcome::Winner(id) => groups.get(&id).unwrap().winner(rules),
        GroupOutcome::RunnerUp(id) => groups.get(&id).unwrap().runner_up(rules),
        GroupOutcome::ThirdPlace(ids) => todo!(),
    };
    (home, away)
}

struct RoundIdx(u8);

#[cfg(test)]
mod tests {
    #[test]
    fn simple_setup() {}
}

#[derive(Error, Debug, Clone, Copy)]
pub enum PlayoffError {
    #[error("Playoff transition id's not a subset of group id's")]
    TransitionGroupIdMismatch,
}
