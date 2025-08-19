use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serializer};
use std::fmt;

pub mod iso8601 {
    use super::*;

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.3fZ";

    pub fn serialize<S>(datetime: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = datetime.format(FORMAT).to_string();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DateTimeVisitor;

        impl<'de> serde::de::Visitor<'de> for DateTimeVisitor {
            type Value = DateTime<Utc>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "an ISO 8601 formatted string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match NaiveDateTime::parse_from_str(value, FORMAT) {
                    Ok(dt) => Ok(Utc.from_utc_datetime(&dt)),
                    Err(_) => Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(value),
                        &"ISO 8601 formatted datetime string",
                    )),
                }
            }
        }

        deserializer.deserialize_str(DateTimeVisitor)
    }
}

pub mod option_iso8601 {
    use super::*;

    pub fn serialize<S>(
        opt_datetime: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match opt_datetime {
            Some(datetime) => super::iso8601::serialize(datetime, serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum DateTimeOrNull {
            DateTime(String),
            Null,
        }

        match DateTimeOrNull::deserialize(deserializer)? {
            DateTimeOrNull::DateTime(s) => {
                let datetime = super::iso8601::deserialize(&s).map_err(serde::de::Error::custom)?;
                Ok(Some(datetime))
            }
            DateTimeOrNull::Null => Ok(None),
        }
    }
}