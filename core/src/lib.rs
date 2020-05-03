use crate::utils::serde_date;
use chrono::{DateTime, FixedOffset, TimeZone};
use serde::{Deserialize, Serialize};
pub mod fair_play;
pub mod game;
pub mod group;
pub mod playoff;
pub mod team;
pub mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub struct Date(#[serde(with = "serde_date")] DateTime<FixedOffset>);

impl Date {
    pub(crate) fn dummy() -> Self {
        let dt = FixedOffset::east(1 * 3600)
            .ymd(1632, 11, 06)
            .and_hms(10, 18, 36);
        Self(dt)
    }
}
