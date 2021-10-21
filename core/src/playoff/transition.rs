use crate::game::GameId;
use crate::group::{GroupId, GroupOutcome};
use crate::playoff::PlayoffError;
use itertools::Itertools;
use std::collections::{BTreeMap, HashSet};
use std::iter::{once, FromIterator};

#[derive(Debug, Clone)]
pub struct PlayoffTransition {
    pub(crate) home: GroupOutcome,
    pub(crate) away: GroupOutcome,
}

// TODO: Optimisation - chained iters instead of allocating hash sets.
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

#[cfg(test)]
pub(crate) mod mock_data {
    use super::*;
    use crate::group::mock_data::groups_and_teams;
    use std::convert::TryFrom;
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
