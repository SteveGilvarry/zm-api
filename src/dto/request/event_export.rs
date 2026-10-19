//! `POST /api/v3/events/export` (#30).

use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// What goes into each event's folder in the zip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ExportPart {
    /// `event.json`: the event as `GET /events/{id}` returns it.
    Details,
    /// `frames.json`: the event's Frames rows.
    Frames,
    /// The event's JPEG files.
    Images,
    /// The event's video files (`.mp4`, `.mkv`, `.webm`, `.avi`).
    Video,
    /// Every other file in the event directory.
    Misc,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct EventExportRequest {
    /// The events to export, at most 500.
    #[garde(length(min = 1, max = 500))]
    pub ids: Vec<u64>,
    /// Defaults to `details` and `video`.
    #[garde(skip)]
    pub include: Option<Vec<ExportPart>>,
}

impl EventExportRequest {
    pub fn parts(&self) -> Vec<ExportPart> {
        self.include
            .clone()
            .unwrap_or_else(|| vec![ExportPart::Details, ExportPart::Video])
    }
}
