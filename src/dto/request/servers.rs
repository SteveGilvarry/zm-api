use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::dto::serde_helpers::double_option;

#[derive(Debug, Default, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateServerRequest {
    #[garde(length(chars, max = 64))]
    pub name: String,
    #[garde(inner(length(chars, max = 255)))]
    #[serde(default)]
    pub hostname: Option<String>,
    #[garde(skip)]
    #[serde(default)]
    pub port: Option<u32>,
    /// `Unknown`, `NotRunning` or `Running`.
    #[garde(skip)]
    #[serde(default)]
    pub status: Option<String>,
    /// `http` or `https`, as the legacy Servers modal offers.
    #[garde(inner(length(chars, max = 16)))]
    #[serde(default)]
    pub protocol: Option<String>,
    #[garde(inner(length(chars, max = 255)))]
    #[serde(default)]
    pub path_to_index: Option<String>,
    #[garde(inner(length(chars, max = 255)))]
    #[serde(default)]
    pub path_to_zms: Option<String>,
    #[garde(inner(length(chars, max = 255)))]
    #[serde(default)]
    pub path_to_api: Option<String>,
    /// Run this daemon on this server.
    #[garde(skip)]
    #[serde(default)]
    pub zmstats: Option<bool>,
    #[garde(skip)]
    #[serde(default)]
    pub zmaudit: Option<bool>,
    #[garde(skip)]
    #[serde(default)]
    pub zmtrigger: Option<bool>,
    #[garde(skip)]
    #[serde(default)]
    pub zmeventnotification: Option<bool>,
    #[garde(inner(range(min = -90.0, max = 90.0)))]
    #[serde(default)]
    pub latitude: Option<f64>,
    #[garde(inner(range(min = -180.0, max = 180.0)))]
    #[serde(default)]
    pub longitude: Option<f64>,
}

/// Partial update: a field left out is unchanged; `null` clears a nullable one.
#[derive(Debug, Default, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateServerRequest {
    #[garde(inner(length(chars, max = 64)))]
    #[serde(default)]
    pub name: Option<String>,
    #[garde(inner(inner(length(chars, max = 255))))]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<String>)]
    pub hostname: Option<Option<String>>,
    #[garde(skip)]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<u32>)]
    pub port: Option<Option<u32>>,
    #[garde(skip)]
    #[serde(default)]
    pub status: Option<String>,
    #[garde(inner(inner(length(chars, max = 16))))]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<String>)]
    pub protocol: Option<Option<String>>,
    #[garde(inner(inner(length(chars, max = 255))))]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<String>)]
    pub path_to_index: Option<Option<String>>,
    #[garde(inner(inner(length(chars, max = 255))))]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<String>)]
    pub path_to_zms: Option<Option<String>>,
    #[garde(inner(inner(length(chars, max = 255))))]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<String>)]
    pub path_to_api: Option<Option<String>>,
    #[garde(skip)]
    #[serde(default)]
    pub zmstats: Option<bool>,
    #[garde(skip)]
    #[serde(default)]
    pub zmaudit: Option<bool>,
    #[garde(skip)]
    #[serde(default)]
    pub zmtrigger: Option<bool>,
    #[garde(skip)]
    #[serde(default)]
    pub zmeventnotification: Option<bool>,
    #[garde(inner(inner(range(min = -90.0, max = 90.0))))]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<f64>)]
    pub latitude: Option<Option<f64>>,
    #[garde(inner(inner(range(min = -180.0, max = 180.0))))]
    #[serde(default, deserialize_with = "double_option")]
    #[schema(value_type = Option<f64>)]
    pub longitude: Option<Option<f64>>,
}
