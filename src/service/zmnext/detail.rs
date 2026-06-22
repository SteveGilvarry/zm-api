//! Typed views over the zm-next EVENT `json_detail` TLV (tag `0x10`).
//!
//! zm-next carries its structured analysis/AI payloads as a UTF-8 JSON document
//! in the EVENT `json_detail` field. The three analysis codes each use a
//! different schema:
//!
//! * `0x0301 detection`   → [`DetectionDetail`]   (object list + source frame)
//! * `0x0302 description` → [`DescriptionDetail`] (the raw `describe_vlm` event)
//! * `0x0303 recording_saved` → [`RecordingSavedDetail`] (the `store_event`
//!   `EventClip` document)
//!
//! Every field is optional / defaulted: zm-next is free to add keys (the wire
//! is additive), so parsing tolerates unknown keys and missing values rather
//! than failing — the same skip-on-unknown discipline as the binary protocol.

use serde::Deserialize;

/// One detected object from a `detection` event.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct DetectedObject {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub confidence: f32,
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default)]
    pub w: f32,
    #[serde(default)]
    pub h: f32,
    /// Persistent tracker id, when the pipeline includes a tracker stage.
    #[serde(default)]
    pub track_id: Option<i64>,
    /// ZoneMinder zone the detection fell in, when zone mapping is enabled.
    #[serde(default)]
    pub zone_id: Option<u32>,
}

/// `detection` (0x0301) `json_detail`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct DetectionDetail {
    #[serde(default)]
    pub objects: Vec<DetectedObject>,
    /// pts (microseconds) of the frame the detection ran on.
    #[serde(default)]
    pub frame_pts_us: Option<i64>,
}

impl DetectionDetail {
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// A short, human-readable cause string ("person, car") for the Event row,
    /// de-duplicated and order-preserving. Empty when there are no objects.
    pub fn cause_summary(&self) -> String {
        let mut labels: Vec<&str> = Vec::new();
        for obj in &self.objects {
            let label = obj.label.trim();
            if !label.is_empty() && !labels.contains(&label) {
                labels.push(label);
            }
        }
        labels.join(", ")
    }

    /// Highest confidence across all objects, scaled to ZoneMinder's 0–100
    /// integer score domain. 0 when there are no objects.
    pub fn peak_score(&self) -> u16 {
        let peak = self
            .objects
            .iter()
            .map(|o| o.confidence)
            .fold(0.0f32, f32::max);
        (peak.clamp(0.0, 1.0) * 100.0).round() as u16
    }
}

/// `description` (0x0302) `json_detail` — the raw `describe_vlm` event.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct DescriptionDetail {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

impl DescriptionDetail {
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// `recording_opening` (0x0304) `json_detail` — emitted when `store_event`
/// begins a segment and needs an event id + target path assigned. `clip_token`
/// is store_event's opaque correlation handle, echoed back in the reply.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct RecordingOpeningDetail {
    #[serde(default)]
    pub clip_token: String,
    #[serde(default)]
    pub trigger: Option<String>,
}

impl RecordingOpeningDetail {
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// `recording_saved` (0x0303) `json_detail` — the `store_event` `EventClip`.
///
/// Only the fields zm-api indexes are modelled; `store_event` may emit more
/// (kept forward-compatible by ignoring unknown keys).
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct RecordingSavedDetail {
    /// Absolute path of the written `.mp4` clip.
    #[serde(default)]
    pub path: String,
    /// The event id zm-api assigned at `recording_opening`, echoed back so the
    /// exact row is finalized. `None` only for clips written without the
    /// handshake (legacy / pre-assignment).
    #[serde(default)]
    pub event_id: Option<u64>,
    /// Clip duration in seconds.
    #[serde(default)]
    pub duration: Option<f64>,
    /// Total frames written, when the muxer reports it.
    #[serde(default)]
    pub frames: Option<u32>,
    /// Unix-epoch seconds of the first / last frame, when present.
    #[serde(default)]
    pub start_time: Option<f64>,
    #[serde(default)]
    pub end_time: Option<f64>,
}

impl RecordingSavedDetail {
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// The clip's file name (what ZoneMinder stores in `Events.DefaultVideo`),
    /// derived from the reported absolute path. Empty when `path` is empty.
    pub fn file_name(&self) -> &str {
        self.path
            .rsplit(['/', '\\'])
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detection_parses_objects_and_summarizes() {
        let json = r#"{
            "objects": [
                {"label":"person","confidence":0.91,"x":10,"y":20,"w":30,"h":40,"track_id":7,"zone_id":2},
                {"label":"car","confidence":0.62},
                {"label":"person","confidence":0.5}
            ],
            "frame_pts_us": 12345
        }"#;
        let d = DetectionDetail::parse(json).unwrap();
        assert_eq!(d.objects.len(), 3);
        assert_eq!(d.objects[0].track_id, Some(7));
        assert_eq!(d.objects[0].zone_id, Some(2));
        assert_eq!(d.frame_pts_us, Some(12345));
        // De-duplicated, order-preserving.
        assert_eq!(d.cause_summary(), "person, car");
        // Peak confidence 0.91 → 91.
        assert_eq!(d.peak_score(), 91);
    }

    #[test]
    fn detection_tolerates_unknown_keys_and_empty() {
        let json = r#"{"objects":[],"model":"yolo26n","extra":{"nested":true}}"#;
        let d = DetectionDetail::parse(json).unwrap();
        assert!(d.objects.is_empty());
        assert_eq!(d.cause_summary(), "");
        assert_eq!(d.peak_score(), 0);
    }

    #[test]
    fn description_parses_text() {
        let json =
            r#"{"text":"A person walks past a parked car.","prompt":"Describe","model":"qwen"}"#;
        let d = DescriptionDetail::parse(json).unwrap();
        assert_eq!(d.text, "A person walks past a parked car.");
        assert_eq!(d.model.as_deref(), Some("qwen"));
    }

    #[test]
    fn recording_saved_parses_path_and_filename() {
        let json = r#"{"event":"EventClip","path":"/var/lib/zm/events/3/2026-06-21/512/512-video.mp4","duration":12.5,"frames":300}"#;
        let r = RecordingSavedDetail::parse(json).unwrap();
        assert_eq!(r.duration, Some(12.5));
        assert_eq!(r.frames, Some(300));
        assert_eq!(r.file_name(), "512-video.mp4");
    }

    #[test]
    fn recording_saved_empty_path_has_no_filename() {
        let r = RecordingSavedDetail::default();
        assert_eq!(r.file_name(), "");
    }

    #[test]
    fn recording_opening_parses_clip_token() {
        let o = RecordingOpeningDetail::parse(r#"{"clip_token":"abc-123","trigger":"detection"}"#)
            .unwrap();
        assert_eq!(o.clip_token, "abc-123");
        assert_eq!(o.trigger.as_deref(), Some("detection"));
    }

    #[test]
    fn recording_saved_echoes_assigned_event_id() {
        let r = RecordingSavedDetail::parse(
            r#"{"event":"EventClip","event_id":512,"path":"/e/3/d/512/512-video.mp4","duration":9.0}"#,
        )
        .unwrap();
        assert_eq!(r.event_id, Some(512));
        assert_eq!(r.file_name(), "512-video.mp4");
    }
}
