pub mod assertion;
pub mod claim;
pub mod dir;
pub mod file;
pub mod hash;
pub mod key;
pub mod middleware;
pub mod password;
pub mod path;
pub mod random;
pub mod regex;
pub mod result;
pub mod retry;
pub mod task;
pub mod ws;

pub mod datetime_format {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    // Format for serializing/deserializing NaiveDateTime
    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.f";

    // For regular NaiveDateTime fields
    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }

    // For Option<NaiveDateTime> fields
    pub mod optional {
        use super::*;

        pub fn serialize<S>(date: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match date {
                Some(date) => super::serialize(date, serializer),
                None => serializer.serialize_none(),
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
        where
            D: Deserializer<'de>,
        {
            Option::<String>::deserialize(deserializer)?
                .map(|s| NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom))
                .transpose()
        }
    }
}
