//! # Group stage to playoff transition
//!
//! The best performing teams in the group stage advance to a playoff.
//! The first round of the playoff is determined by all groups and a set of transition rules.
//! Transition rules differ across tournaments
//! Currently supported is to have winners, runner ups and top third placers to advance.
use crate::game::GameId;
use crate::group::order::{Rules, Tiebreaker};
use crate::group::{GroupId, GroupOutcome, Groups};
use crate::playoff::PlayoffError;
use crate::team::TeamId;
use itertools::Itertools;
use std::collections::{BTreeMap, HashSet};

/// Single transition
///
/// Describes how one game in the first round should be populated
#[derive(Debug, Clone)]
pub struct PlayoffTransition {
    pub(crate) home: GroupOutcome,
    pub(crate) away: GroupOutcome,
}

// TODO: Optimisation - chained iters instead of allocating hash sets.
impl PlayoffTransition {
    fn group_ids(&self) -> HashSet<GroupId> {
        let home: HashSet<GroupId> = match &self.home {
            GroupOutcome::Winner(id) => HashSet::from([*id]),
            GroupOutcome::RunnerUp(id) => HashSet::from([*id]),
            GroupOutcome::ThirdPlace(ids) => ids.clone(),
        };
        let away: HashSet<GroupId> = match &self.away {
            GroupOutcome::Winner(id) => HashSet::from([*id]),
            GroupOutcome::RunnerUp(id) => HashSet::from([*id]),
            GroupOutcome::ThirdPlace(ids) => ids.clone(),
        };
        home.union(&away).cloned().collect()
    }
}

impl PlayoffTransition {
    pub fn new(home: GroupOutcome, away: GroupOutcome) -> Self {
        Self { home, away }
    }
}

/// Transition collection
///
/// Mapping from [`GameId`] to [`PlayoffTransition`]
#[derive(Debug, Clone)]
pub struct PlayoffTransitions(BTreeMap<GameId, PlayoffTransition>);

impl PlayoffTransitions {
    /// Fallible constructor for transitions
    ///
    /// # Errors
    ///
    /// Constructor fails if there is a mismatch between groups and transitions.
    /// The set of id's in the transitions must be a subset of the groups' id's
    pub fn try_new(
        transitions: impl Iterator<Item = (GameId, PlayoffTransition)>,
        groups: &HashSet<GroupId>,
    ) -> Result<Self, PlayoffError> {
        // Copy the iterator, one for the set check and one for the actual constructor.
        let (transitions, check) = transitions.tee();
        let trans_ids = check
            .map(|(_, trans)| trans)
            .fold(HashSet::new(), |acc, trans| {
                acc.union(&trans.group_ids()).cloned().collect()
            });
        if trans_ids.is_subset(groups) {
            Ok(PlayoffTransitions(
                transitions.collect::<BTreeMap<GameId, PlayoffTransition>>(),
            ))
        } else {
            Err(PlayoffError::TransitionGroupIdMismatch)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&GameId, &PlayoffTransition)> {
        self.0.iter()
    }
}

/// Resolve a team from a group outcome
///
/// This is used by the bracket system to resolve teams from the group stage.
pub fn resolve_from_group_outcome<T: Tiebreaker>(
    groups: &Groups,
    outcome: &GroupOutcome,
    rules: &Rules<T>,
) -> TeamId {
    match outcome {
        GroupOutcome::Winner(group_id) => groups.get(group_id).unwrap().winner(rules),
        GroupOutcome::RunnerUp(group_id) => groups.get(group_id).unwrap().runner_up(rules),
        GroupOutcome::ThirdPlace(group_ids) => {
            // For third place, we need special third-place rules
            // For now, just take the first group's third place
            // TODO: Implement proper third-place ranking
            let first_group = group_ids.iter().next().unwrap();
            groups.get(first_group).unwrap().third_place(rules)
        }
    }
}

#[cfg(test)]
pub(crate) mod mock_data {
    use super::*;
    use crate::group::mock_data::groups_and_teams;
    pub(crate) fn transitions() -> PlayoffTransitions {
        let id_a = GroupId::try_from('A').unwrap();
        let id_b = GroupId::try_from('B').unwrap();
        let game_1 = PlayoffTransition {
            home: GroupOutcome::Winner(id_a),
            away: GroupOutcome::ThirdPlace(HashSet::from_iter([id_a, id_b].iter().cloned())),
        };
        let game_2 = PlayoffTransition {
            home: GroupOutcome::Winner(id_b),
            away: GroupOutcome::ThirdPlace(HashSet::from_iter([id_a, id_b].iter().cloned())),
        };

        let (groups, _) = groups_and_teams();
        PlayoffTransitions::try_new(
            [1, 2]
                .iter()
                .copied()
                .map(GameId::from)
                .zip([game_1, game_2].iter().cloned()),
            &groups.keys().copied().collect(),
        )
        .unwrap()
    }
}
