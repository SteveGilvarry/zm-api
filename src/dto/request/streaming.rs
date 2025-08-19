use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request payload to register a ZoneMinder stream with go2rtc
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterStreamRequest {
    pub zm_user: String,
    pub zm_pass: String,
    pub zm_host: String,
    pub zm_port: u16,
}