use crate::game::GameId;
use crate::group::{GroupId, GroupOutcome, Groups};
use crate::playoff::PlayoffError;
use itertools::Itertools;
use std::collections::{BTreeMap, HashSet};
use std::iter::{once, FromIterator};

#[derive(Debug, Clone)]
pub struct PlayoffTransition {
    home: GroupOutcome,
    away: GroupOutcome,
}

impl PlayoffTransition {
    fn group_ids(&self) -> HashSet<GroupId> {
        let home: HashSet<GroupId> = match &self.home {
            GroupOutcome::Winner(id) => HashSet::from_iter(once(*id)),
            GroupOutcome::RunnerUp(id) => HashSet::from_iter(once(*id)),
            GroupOutcome::ThirdPlace(ids) => ids.clone(),
        };
        let away: HashSet<GroupId> = match &self.away {
            GroupOutcome::Winner(id) => HashSet::from_iter(once(*id)),
            GroupOutcome::RunnerUp(id) => HashSet::from_iter(once(*id)),
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
