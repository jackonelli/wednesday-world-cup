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

pub struct Playoff {
    rounds: HashMap<RoundIdx, Round>,
}

pub struct Round {
    pub games: Vec<PlayoffGame>,
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
        .collect::<HashMap<TeamId, &Group>>();
    order_teams(&candidates, third_place_rules)[OrderIdx(0)]
}

struct RoundIdx(u8);

#[derive(Error, Debug, Clone, Copy)]
pub enum PlayoffError {
    #[error("Playoff transition id's not a subset of group id's")]
    TransitionGroupIdMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group::mock_data::groups_and_teams;
    use crate::group::order::{euro_2020, euro_2020_third_place, UefaRanking};
    use crate::playoff::transition::mock_data::transitions;
    #[test]
    fn mock_data_access() {
        let (mock_groups, mock_teams) = groups_and_teams();
        let mock_trans = transitions();
        let ranking = UefaRanking::try_new(
            &mock_groups,
            mock_teams
                .iter()
                .map(|(id, team)| (*id, team.rank))
                .collect(),
        )
        .unwrap();
        let round = Round::first_round_from_group_stage(
            &mock_groups,
            &mock_trans,
            &euro_2020(ranking.clone()),
            &euro_2020_third_place(ranking),
        );
        assert_eq!(round.games[0].home.unwrap(), TeamId::from(1));
        assert_eq!(round.games[0].away.unwrap(), TeamId::from(4));
    }
}
