use crate::group::Group;
use crate::team::TeamId;

// TODO: Struct best way? Perhaps alias trait Fn: Group -> GroupOrder?
pub trait Order {
    fn order(&self, group: &Group) -> GroupOrder;
    fn winner(&self, group: &Group) -> TeamId {
        self.order(group).winner()
    }
    fn runner_up(&self, group: &Group) -> TeamId {
        self.order(group).runner_up()
    }
}

/// FIFA World Cup 2018 Order
///
/// https://www.fifa.com/worldcup/news/tie-breakers-for-russia-2018-groups
///
///First step: Pursuant to the criteria listed in art. 32 (5) lit. a) to c) of the Competition Regulations.
///
///1. Greatest number of points obtained in all group matches
///2. Goal difference in all group matches
///3. Greatest number of goals scored in all group matches.
///
///Second step: If two or more teams are equal on the basis of the first step (see example in Table 1), their ranking will be determined by applying to the group matches between the teams concerned the criteria listed in art. 32 (5) lit. d) to h) in the order of their listing.
///
/// 4. Greatest number of points obtained in the group matches between the teams concerned
/// 5. Goal difference resulting from the group matches between the teams concerned
/// 6. Greater number of goals scored in all group matches between the teams concerned
/// 7. Greater number of points obtained in the fair play conduct of the teams based on yellow and red cards received in all group matches
///     - Yellow card: –1 points
///     - Indirect red card (second yellow card): –3 points
///     - Direct red card: –4 points
///     - Yellow card and direct red card: –5 points
/// 8. Drawing of lots by the FIFA.
pub struct Fifa2018Order {}

#[derive(Clone, Copy, Debug)]
pub struct GroupRank(usize);

pub struct GroupOrder(Vec<TeamId>);

impl GroupOrder {
    pub fn winner(&self) -> TeamId {
        self[GroupRank(0)]
    }
    pub fn runner_up(&self) -> TeamId {
        self[GroupRank(1)]
    }
}

impl std::ops::Index<GroupRank> for GroupOrder {
    type Output = TeamId;
    fn index(&self, idx: GroupRank) -> &Self::Output {
        &self.0[idx.0]
    }
}
