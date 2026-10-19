//! Requests for the media job queue (#187).

use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// A filter for the generated video, as ZoneMinder's Generate Video offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum VideoTransform {
    #[default]
    None,
    /// ffmpeg's `hue=s=0`.
    Greyscale,
}

/// `POST /api/v3/events/{id}/videos`: what zmvideo.pl's options set.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema, Validate)]
pub struct GenerateVideoRequest {
    /// One of `ZM_FFMPEG_FORMATS`; defaults to its starred entry.
    #[garde(inner(length(chars, min = 1, max = 16)))]
    pub format: Option<String>,
    /// Playback speed relative to real time (2 is twice as fast). Exclusive
    /// with `fps`.
    #[garde(inner(range(min = 0.01, max = 100.0)))]
    pub rate: Option<f64>,
    /// A fixed output frame rate. Exclusive with `rate`.
    #[garde(inner(range(min = 0.1, max = 120.0)))]
    pub fps: Option<f64>,
    /// Scale relative to the event's size. Exclusive with `size`.
    #[garde(inner(range(min = 0.05, max = 4.0)))]
    pub scale: Option<f64>,
    /// An exact `WIDTHxHEIGHT`. Exclusive with `scale`.
    #[garde(inner(pattern(r"^[1-9][0-9]{0,4}x[1-9][0-9]{0,4}$")))]
    pub size: Option<String>,
    #[garde(skip)]
    #[serde(default)]
    pub transform: VideoTransform,
    /// Replace a file of the same name; otherwise an existing one is the result.
    #[garde(skip)]
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct JobListQuery {
    pub event_id: Option<u64>,
    /// `queued`, `running`, `done`, `failed` or `cancelled`.
    pub status: Option<String>,
}
