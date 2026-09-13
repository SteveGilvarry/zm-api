//! Responses for a zm-next monitor's on-demand worker commands.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A scene description produced on request by the monitor's `describe_vlm`.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct DescribeNowResponse {
    pub monitor_id: u32,
    /// The model's description.
    pub text: String,
    /// Prompt the model was given.
    pub prompt: Option<String>,
    pub model: Option<String>,
    /// Frames sent to the model.
    pub frames: Option<u32>,
    /// Presentation timestamp of the described frame, microseconds.
    pub pts_usec: Option<i64>,
    pub stream_id: Option<u32>,
}
