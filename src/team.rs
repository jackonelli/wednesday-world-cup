use derive_more::From;
#[derive(Debug, Clone, Copy, std::cmp::Eq, std::cmp::PartialEq, std::hash::Hash, From)]
pub struct TeamId(pub u8);
