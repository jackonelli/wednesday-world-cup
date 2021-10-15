//! Top-level error type for wwc_core
use crate::group::GroupError;
use crate::team::TeamError;
use thiserror::Error;

/// Top-level error type for wwc_core
#[derive(Error, Debug, Clone)]
pub enum WwcError {
    #[error("Group error: {0}")]
    Group(#[from] GroupError),
    #[error("Team error: {0}")]
    Team(#[from] TeamError),
}
