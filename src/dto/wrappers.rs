use crate::entity::sea_orm_active_enums::Scheme;
use chrono::{DateTime, NaiveDateTime, Utc};
use fake::Dummy;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Dummy, Clone)]
#[schema(value_type = String, format = "date-time", example = "2025-04-24T12:34:56Z")]
pub struct DateTimeWrapper(pub DateTime<Utc>);

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[schema(value_type = String, format = "date-time", example = "2025-04-24T12:34:56")]
pub struct NaiveDateTimeWrapper(pub NaiveDateTime);

// Add conversion implementations for NaiveDateTimeWrapper
impl From<NaiveDateTimeWrapper> for NaiveDateTime {
    fn from(wrapper: NaiveDateTimeWrapper) -> Self {
        wrapper.0
    }
}

impl From<NaiveDateTime> for NaiveDateTimeWrapper {
    fn from(dt: NaiveDateTime) -> Self {
        NaiveDateTimeWrapper(dt)
    }
}

// Add reference conversion implementations
impl<'a> From<&'a NaiveDateTimeWrapper> for &'a NaiveDateTime {
    fn from(wrapper: &'a NaiveDateTimeWrapper) -> Self {
        &wrapper.0
    }
}

// Helper function to convert Option<NaiveDateTimeWrapper> to Option<NaiveDateTime>
pub fn unwrap_naive_datetime_wrapper(
    opt_wrapper: &Option<NaiveDateTimeWrapper>,
) -> Option<NaiveDateTime> {
    opt_wrapper.as_ref().map(|wrapper| wrapper.0)
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[schema(value_type = String, example = "123.45")]
pub struct DecimalWrapper(pub Decimal);

impl From<Decimal> for DecimalWrapper {
    fn from(d: Decimal) -> Self {
        DecimalWrapper(d)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[schema(value_type = String, example = "Deep")]
pub struct SchemeWrapper(pub Scheme);

impl From<Scheme> for SchemeWrapper {
    fn from(s: Scheme) -> Self {
        SchemeWrapper(s)
    }
}

impl From<DateTime<Utc>> for DateTimeWrapper {
    fn from(dt: DateTime<Utc>) -> Self {
        DateTimeWrapper(dt)
    }
}

/// Serde support for Option<NaiveDateTimeWrapper> using existing optional formatting
pub mod naive_datetime_option {
    use super::NaiveDateTimeWrapper;
    use crate::util::datetime_format::optional as base;
    use serde::{self, Deserializer, Serializer};
    // use chrono::NaiveDateTime; // Unused import

    pub fn serialize<S>(
        val: &Option<NaiveDateTimeWrapper>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Explicitly handle Option and serialize the inner NaiveDateTime
        match val {
            Some(wrapper) => base::serialize(&Some(wrapper.0), serializer),
            None => base::serialize::<S>(&None, serializer), // Specify type for None case
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTimeWrapper>, D::Error>
    where
        D: Deserializer<'de>,
    {
        base::deserialize(deserializer).map(|opt| opt.map(NaiveDateTimeWrapper))
    }
}
