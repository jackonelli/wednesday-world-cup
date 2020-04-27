use crate::group::Group;
use crate::team::TeamId;

pub trait Order {
    fn order(&self, group: &Group) -> GroupOrder;
    fn winner(&self, group: &Group) -> TeamId {
        self.order(group).winner()
    }
    fn runner_up(&self, group: &Group) -> TeamId {
        self.order(group).runner_up()
    }
}

pub struct GroupOrder(Vec<TeamId>);

impl GroupOrder {
    fn team_by_rank(&self, rank: GroupRank) -> TeamId {
        // TODO impl slice for rank: GroupOrder[GroupRank] -> TeamId
        (self.0)[rank.0]
    }
    pub fn winner(&self) -> TeamId {
        self.team_by_rank(GroupRank(0))
    }
    pub fn runner_up(&self) -> TeamId {
        self.team_by_rank(GroupRank(1))
    }
}

pub struct GroupRank(usize);
