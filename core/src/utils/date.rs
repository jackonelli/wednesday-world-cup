//! Date
use chrono::{DateTime, FixedOffset, TimeZone};
use serde::{self, Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
pub struct Date(#[serde(with = "serde_date")] DateTime<FixedOffset>);

impl Date {
    pub fn mock() -> Self {
        let dt = FixedOffset::east(3600).ymd(1632, 11, 6).and_hms(10, 18, 36);
        Self(dt)
    }
}

mod serde_date {
    use chrono::{DateTime, FixedOffset};
    use serde::{self, Deserialize, Deserializer, Serializer};
    /// The signature of a serialize_with function must follow the pattern:
    ///
    ///```not_rust
    ///fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    ///where
    ///    S: Serializer
    ///```
    /// although it may also be generic over the input types T.
    pub fn serialize<S>(date: &DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        //let s = format!("{}", date.format(FORMAT));
        let s = date.to_rfc3339();
        serializer.serialize_str(&s)
    }

    /// The signature of a deserialize_with function must follow the pattern:
    ///
    ///```not_rust
    ///fn deserialize<'de, D>(D) -> Result<T, D::Error>
    ///where
    ///    D: Deserializer<'de>
    ///```
    /// although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)
    }
}
