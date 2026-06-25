//! Typed views over the zm-next EVENT `json_detail` TLV (tag `0x10`).
//!
//! zm-next carries its structured analysis/AI payloads as a UTF-8 JSON document
//! in the EVENT `json_detail` field. The three analysis codes each use a
//! different schema:
//!
//! * `0x0301 detection`   → [`DetectionDetail`]   (object list + source frame)
//! * `0x0302 description` → [`DescriptionDetail`] (the raw `describe_vlm` event)
//! * `0x0304 recording_opening` → [`RecordingOpeningDetail`] (id-assignment request)
//! * `0x0303 recording_saved` → [`RecordingSavedDetail`] (the `store` plugin's
//!   `EventClip` document)
//!
//! Every field is optional / defaulted: zm-next is free to add keys (the wire
//! is additive), so parsing tolerates unknown keys and missing values rather
//! than failing — the same skip-on-unknown discipline as the binary protocol.

use std::path::{Path, PathBuf};

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

/// `recording_opening` (0x0304) `json_detail` — emitted when the `store` plugin
/// begins a segment and needs an event id + target path assigned. `clip_token`
/// is its opaque correlation handle, echoed back in the reply. `trigger` is
/// `"continuous"` for a continuous segment, else the trigger type
/// (`"detection"`, `"motion"`, `"audio_event"`, `"tracked_detection"`, …).
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

/// `recording_saved` (0x0303) `json_detail` — the `store` plugin's `EventClip`.
///
/// Only the fields zm-api indexes are modelled; the worker may emit more
/// (kept forward-compatible by ignoring unknown keys).
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct RecordingSavedDetail {
    /// Absolute path of the written `.mp4` clip (the assigned target if the
    /// rename succeeded, else the worker's own-naming path).
    #[serde(default)]
    pub path: String,
    /// The event id zm-api assigned at `recording_opening`, echoed back so the
    /// exact row is finalized. `0` (or absent) means the clip closed before the
    /// assignment arrived — see [`Self::assigned_event_id`].
    #[serde(default)]
    pub event_id: Option<u64>,
    /// Cause string the worker recorded the clip under.
    #[serde(default)]
    pub cause: Option<String>,
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

    /// The assigned event id, treating `0`/absent as "not assigned" (the clip
    /// closed before zm-api's `assign_recording` reached the worker).
    pub fn assigned_event_id(&self) -> Option<u64> {
        self.event_id.filter(|&id| id != 0)
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

// ============================================================================
// review_assets (0x0306) — the motion-synopsis manifest
// ============================================================================

/// `review_assets` (0x0306) `json_detail` — the motion-synopsis manifest.
///
/// zm-next emits the "ingredients" for a synopsis (object *tubes* of
/// pre-rendered cutout JPEGs plus background *plates*) and references them by
/// path here. zm-api never re-decodes the clip: every pixel it composites comes
/// from the cutout/plate JPEGs this manifest points at. The discriminator is
/// `type` (`"review_assets"`), matching zm-next's `WorkerLink::map_event_code`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct TubeManifest {
    #[serde(default, rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub schema: u32,
    #[serde(default)]
    pub monitor_id: u32,
    /// `0` when the recording_opening→assign handshake didn't finish; fall back
    /// to `clip_token` as the correlation key in that case.
    #[serde(default)]
    pub event_id: u64,
    #[serde(default)]
    pub clip_token: String,
    /// Absolute path of the recorded clip the assets were derived from.
    #[serde(default)]
    pub clip_path: String,
    /// Asset directory holding cutouts/plates: relative to `dirname(clip_path)`,
    /// or absolute when the clip kept the store's own naming (rename failed).
    #[serde(default)]
    pub path_base: String,
    /// Event media-clock span, microseconds.
    #[serde(default)]
    pub t_start_us: i64,
    #[serde(default)]
    pub t_end_us: i64,
    /// Coordinate space for every bbox / polygon / cutout in this manifest.
    #[serde(default)]
    pub source_w: u32,
    #[serde(default)]
    pub source_h: u32,
    #[serde(default)]
    pub sample_fps: f32,
    #[serde(default)]
    pub plates: Vec<Plate>,
    #[serde(default)]
    pub tubes: Vec<Tube>,
}

/// A background plate (clean frame) at its own resolution. The renderer must
/// rescale it to `source_w × source_h` before compositing.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Plate {
    /// Path relative to `path_base` (or absolute).
    #[serde(default)]
    pub path: String,
    /// Wall-clock time of the plate, ms — used to pick the time-nearest plate.
    #[serde(default)]
    pub wallclock_ms: i64,
    #[serde(default)]
    pub w: u32,
    #[serde(default)]
    pub h: u32,
    /// Illumination hint ("day"/"night"); advisory.
    #[serde(default)]
    pub illum: Option<String>,
}

/// One object track ("tube"): a time-ordered run of cutout samples.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Tube {
    #[serde(default)]
    pub track_id: i64,
    #[serde(default)]
    pub label: String,
    /// Detector class id — used for class filtering (e.g. people-only synopsis).
    #[serde(default)]
    pub class_id: i64,
    #[serde(default)]
    pub t_start_us: i64,
    #[serde(default)]
    pub t_end_us: i64,
    #[serde(default)]
    pub samples: Vec<TubeSample>,
}

/// One sampled cutout for a tube. `bbox` is `[x, y, w, h]` in source coords.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct TubeSample {
    #[serde(default)]
    pub pts_us: i64,
    #[serde(default)]
    pub wallclock_ms: i64,
    /// `[x, y, w, h]` in source coords.
    #[serde(default)]
    pub bbox: [i32; 4],
    /// Premultiplied-RGB cutout JPEG (background → black), relative to
    /// `path_base`.
    #[serde(default)]
    pub cutout: String,
    #[serde(default)]
    pub cutout_w: u32,
    #[serde(default)]
    pub cutout_h: u32,
    /// Optional alpha-edge mask. Absent → use the premultiplied cutout as-is.
    #[serde(default)]
    pub mask: Option<Mask>,
}

/// A per-sample alpha mask. `polygon` points are in **source** coords (offset by
/// `bbox`); `rle` counts are **bbox-local**.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "format", rename_all = "lowercase")]
pub enum Mask {
    Polygon {
        #[serde(default)]
        points: Vec<[f32; 2]>,
    },
    Rle {
        #[serde(default)]
        w: u32,
        #[serde(default)]
        h: u32,
        #[serde(default)]
        counts: Vec<u32>,
    },
    /// Forward-compat: an unrecognised `format` degrades to "no mask" rather
    /// than failing the whole manifest parse.
    #[serde(other)]
    Unknown,
}

impl TubeManifest {
    pub fn parse(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// True iff this is the expected discriminator.
    pub fn is_review_assets(&self) -> bool {
        self.kind == "review_assets"
    }

    /// The assigned event id, treating `0`/absent as "not assigned".
    pub fn assigned_event_id(&self) -> Option<u64> {
        (self.event_id != 0).then_some(self.event_id)
    }

    /// Total cutout samples across all tubes (cheap manifest-size metric).
    pub fn total_samples(&self) -> usize {
        self.tubes.iter().map(|t| t.samples.len()).sum()
    }

    /// Resolve the absolute asset directory holding the cutouts/plates:
    /// `path_base` as-is when absolute, else `dirname(clip_path)/path_base`.
    /// `None` when `path_base` is empty, or relative with no usable clip dir.
    pub fn asset_dir(&self) -> Option<PathBuf> {
        if self.path_base.is_empty() {
            return None;
        }
        let base = Path::new(&self.path_base);
        if base.is_absolute() {
            return Some(base.to_path_buf());
        }
        let parent = Path::new(&self.clip_path).parent()?;
        if parent.as_os_str().is_empty() {
            return None;
        }
        Some(parent.join(base))
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

    // --- review_assets (TubeManifest) ---------------------------------------

    const SAMPLE_MANIFEST: &str = r#"{
        "type": "review_assets", "schema": 1,
        "monitor_id": 3, "event_id": 512, "clip_token": "3-1782129185-7",
        "clip_path": "/data/3/2026-06-25/512/512-video.mkv",
        "path_base": "synopsis",
        "t_start_us": 1782129185000000, "t_end_us": 1782129260000000,
        "source_w": 1280, "source_h": 720, "sample_fps": 4,
        "plates": [
            {"path":"plate-1782129180.jpg","wallclock_ms":1782129180000,"w":640,"h":360,"illum":"day"}
        ],
        "tubes": [
            {"track_id":17,"label":"person","class_id":0,
             "t_start_us":1782129186000000,"t_end_us":1782129200000000,
             "samples":[
                {"pts_us":1782129186250000,"wallclock_ms":1782129186250,
                 "bbox":[840,120,96,220],"cutout":"t17/000001.jpg",
                 "cutout_w":96,"cutout_h":220,
                 "mask":{"format":"polygon","points":[[840,120],[936,120],[936,340]]}}
             ]}
        ]
    }"#;

    #[test]
    fn manifest_parses_fully() {
        let m = TubeManifest::parse(SAMPLE_MANIFEST).unwrap();
        assert!(m.is_review_assets());
        assert_eq!(m.monitor_id, 3);
        assert_eq!(m.assigned_event_id(), Some(512));
        assert_eq!(m.clip_token, "3-1782129185-7");
        assert_eq!(m.source_w, 1280);
        assert_eq!(m.plates.len(), 1);
        assert_eq!(m.plates[0].wallclock_ms, 1782129180000);
        assert_eq!(m.tubes.len(), 1);
        assert_eq!(m.tubes[0].track_id, 17);
        assert_eq!(m.tubes[0].label, "person");
        assert_eq!(m.total_samples(), 1);
        let s = &m.tubes[0].samples[0];
        assert_eq!(s.bbox, [840, 120, 96, 220]);
        assert_eq!(s.cutout, "t17/000001.jpg");
        match s.mask.as_ref().unwrap() {
            Mask::Polygon { points } => assert_eq!(points.len(), 3),
            other => panic!("expected polygon, got {other:?}"),
        }
    }

    #[test]
    fn manifest_resolves_relative_asset_dir_under_clip_dir() {
        let m = TubeManifest::parse(SAMPLE_MANIFEST).unwrap();
        assert_eq!(
            m.asset_dir().unwrap(),
            PathBuf::from("/data/3/2026-06-25/512/synopsis")
        );
    }

    #[test]
    fn manifest_keeps_absolute_path_base() {
        let json =
            r#"{"type":"review_assets","clip_path":"/a/b/c.mkv","path_base":"/srv/assets/x"}"#;
        let m = TubeManifest::parse(json).unwrap();
        assert_eq!(m.asset_dir().unwrap(), PathBuf::from("/srv/assets/x"));
    }

    #[test]
    fn manifest_event_id_zero_falls_back_to_token() {
        let json = r#"{"type":"review_assets","event_id":0,"clip_token":"tok-1"}"#;
        let m = TubeManifest::parse(json).unwrap();
        assert_eq!(m.assigned_event_id(), None);
        assert_eq!(m.clip_token, "tok-1");
    }

    #[test]
    fn manifest_rle_mask_and_unknown_format() {
        let rle = r#"{"type":"review_assets","tubes":[{"samples":[
            {"mask":{"format":"rle","w":4,"h":2,"counts":[2,2,2,2]}}]}]}"#;
        let m = TubeManifest::parse(rle).unwrap();
        match m.tubes[0].samples[0].mask.as_ref().unwrap() {
            Mask::Rle { w, h, counts } => {
                assert_eq!((*w, *h), (4, 2));
                assert_eq!(counts, &[2, 2, 2, 2]);
            }
            other => panic!("expected rle, got {other:?}"),
        }
        // An unrecognised mask format degrades to Unknown, not a parse failure.
        let weird = r#"{"type":"review_assets","tubes":[{"samples":[
            {"mask":{"format":"quadtree","blob":"..."}}]}]}"#;
        let m = TubeManifest::parse(weird).unwrap();
        assert_eq!(m.tubes[0].samples[0].mask, Some(Mask::Unknown));
    }

    #[test]
    fn manifest_tolerates_unknown_keys_and_empty() {
        let m = TubeManifest::parse(r#"{"type":"review_assets","future":{"x":1}}"#).unwrap();
        assert!(m.is_review_assets());
        assert!(m.tubes.is_empty());
        assert!(m.asset_dir().is_none()); // no path_base
    }
}
