use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Mint a signed playlist link.
#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, Validate)]
pub struct CreateMediaLinkRequest {
    /// Lifetime in seconds, 60 to 31536000 (365 days). Default 2592000
    /// (30 days).
    #[garde(inner(range(min = 60, max = 31_536_000)))]
    pub ttl_seconds: Option<u64>,
}
