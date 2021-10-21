//! Tournament playoff
pub mod game;
pub mod transition;
use crate::group::order::{order_teams, OrderIdx, Rules, Tiebreaker};
use crate::group::{Group, GroupId, GroupOutcome, Groups};
use crate::playoff::game::PlayoffGame;
use crate::playoff::transition::{PlayoffTransition, PlayoffTransitions};
use crate::team::TeamId;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

struct Playoff {
    rounds: HashMap<RoundIdx, Round>,
}

struct Round {
    games: Vec<PlayoffGame>,
}

impl Round {
    pub fn first_round_from_group_stage<T: Tiebreaker, U: Tiebreaker>(
        groups: &Groups,
        transitions: &PlayoffTransitions,
        group_rules: &Rules<T>,
        third_place_rules: &Rules<U>,
    ) -> Self {
        Self {
            games: transitions
                .iter()
                .map(|(id, trans)| {
                    let (home, away) =
                        teams_from_groups(groups, trans, group_rules, third_place_rules);
                    PlayoffGame::new(*id, home, away)
                })
                .collect(),
        }
    }
}

fn teams_from_groups<T: Tiebreaker, U: Tiebreaker>(
    groups: &Groups,
    trans: &PlayoffTransition,
    group_rules: &Rules<T>,
    third_place_rules: &Rules<U>,
) -> (TeamId, TeamId) {
    let home = match &trans.home {
        GroupOutcome::Winner(id) => groups.get(&id).unwrap().winner(group_rules),
        GroupOutcome::RunnerUp(id) => groups.get(&id).unwrap().runner_up(group_rules),
        GroupOutcome::ThirdPlace(ids) => {
            best_third_place(groups, ids, group_rules, third_place_rules)
        }
    };
    let away = match &trans.away {
        GroupOutcome::Winner(id) => groups.get(&id).unwrap().winner(group_rules),
        GroupOutcome::RunnerUp(id) => groups.get(&id).unwrap().runner_up(group_rules),
        GroupOutcome::ThirdPlace(ids) => {
            best_third_place(groups, ids, group_rules, third_place_rules)
        }
    };
    (home, away)
}

fn best_third_place<T: Tiebreaker, U: Tiebreaker>(
    groups: &Groups,
    group_ids: &HashSet<GroupId>,
    group_rules: &Rules<T>,
    third_place_rules: &Rules<U>,
) -> TeamId {
    let candidates = group_ids
        .iter()
        .map(|id| {
            let group = groups.get(id).unwrap();
            (group.third_place(group_rules), group)
        })
        .collect::<Vec<(TeamId, &Group)>>();
    order_teams(&candidates, third_place_rules)[OrderIdx(0)]
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
