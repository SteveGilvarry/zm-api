use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A signed playlist link for a player that can only authenticate in the URL.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MediaLinkResponse {
    /// The playlist path with its signature, ready for a player: prefix it
    /// with the server's origin. The playlist signs every URI it lists.
    #[schema(example = "/api/v3/live/1/hls/master.m3u8?st=1.m.1.3.1760000000.1762592000.Q2…")]
    pub playlist_url: String,
    /// The signature alone (`st`), for building other URLs of the same
    /// monitor or event.
    pub token: String,
    /// Unix time the link stops working.
    pub expires_at: i64,
}
