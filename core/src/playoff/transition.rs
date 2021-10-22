//! # Group stage to playoff transition
//!
//! The best performing teams in the group stage advance to a playoff.
//! The first round of the playoff is determined by all groups and a set of transition rules.
//! Transition rules differ across tournaments
//! Currently supported is to have winners, runner ups and top third placers to advance.
use crate::game::GameId;
use crate::group::{GroupId, GroupOutcome};
use crate::playoff::PlayoffError;
use itertools::Itertools;
use std::collections::{BTreeMap, HashSet};

/// Single transition
///
/// Describes how one game in the first round should be populated
#[derive(Debug, Clone)]
pub struct PlayoffTransition {
    home: GroupOutcome,
    away: GroupOutcome,
}

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
    pub fn try_new(
        transitions: impl Iterator<Item = (GameId, PlayoffTransition)>,
        groups: &HashSet<GroupId>,
    ) -> Result<Self, PlayoffError> {
        let (transitions, check) = transitions.tee();
        let trans_ids = check
            .map(|(_, trans)| trans)
            .fold(HashSet::new(), |acc, trans| {
                acc.union(&trans.group_ids()).cloned().collect()
            });
        if trans_ids.is_subset(groups) {
            Ok(PlayoffTransitions(
                transitions
                    .collect::<BTreeMap<GameId, PlayoffTransition>>()
                    .clone(),
            ))
        } else {
            println!("Transitions: {:?}", trans_ids);
            println!("Group: {:?}", groups);
            Err(PlayoffError::TransitionGroupIdMismatch)
        }
    }
}
