//! Player
//!
//! Player/User/Better representation
use std::collections::BTreeMap;

pub struct Player {
    name: String,
    id: PlayerId,
    // preds: BTreeMap,
}

/// Numeric player id for db.
pub struct PlayerId(i32);
