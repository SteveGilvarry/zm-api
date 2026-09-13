//! Requests for a zm-next monitor's on-demand worker commands.

use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// `POST /api/v3/monitors/{monitor_id}/snapshot` query.
#[derive(Debug, Clone, Default, Deserialize, Serialize, IntoParams, ToSchema)]
pub struct SnapshotNowQuery {
    /// Worker stream to snapshot. Omit to use whichever instance answers.
    pub stream_id: Option<u32>,
}

/// `POST /api/v3/monitors/{monitor_id}/describe` body. Every field is optional.
#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, Validate)]
pub struct DescribeNowRequest {
    /// One-off prompt replacing the pipeline's configured one for this call.
    #[garde(inner(length(chars, max = 2000)))]
    pub prompt: Option<String>,
    /// Worker stream to describe. Omit to use whichever instance answers.
    #[garde(skip)]
    pub stream_id: Option<u32>,
}
