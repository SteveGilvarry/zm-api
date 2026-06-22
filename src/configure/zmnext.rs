//! Configuration for the zm-next worker integration.
//!
//! Covers the three zm-api-side concerns of driving a zm-next worker:
//!   * spawning the worker binary (`[zmnext.worker]`),
//!   * generating its pipeline JSON from the Monitors/Zones rows
//!     (`[zmnext.pipeline]`),
//!   * ingesting the analysis EVENTs it emits back into Events/Frames
//!     (`[zmnext.ingest]`).
//!
//! The whole section defaults to a disabled, zero-impact configuration: with no
//! `[zmnext]` block, `enabled` is false and the daemon keeps spawning legacy
//! `zmc`/`zma` exactly as before.

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ZmNextConfig {
    /// Master switch. When false, no monitor is routed to a zm-next worker
    /// regardless of its per-monitor flag, and the ingest task is not spawned.
    pub enabled: bool,
    pub worker: WorkerConfig,
    pub pipeline: PipelineConfig,
    pub ingest: IngestConfig,
}

/// How to spawn and supervise the per-monitor zm-next worker process.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct WorkerConfig {
    /// Worker executable. Resolved on `PATH` when not absolute.
    pub binary: String,
}

/// Inputs to the pipeline-JSON generator.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PipelineConfig {
    /// Directory the generated `monitor_{id}.json` pipeline files are written
    /// to (and passed to the worker via `--pipeline`).
    pub dir: PathBuf,
    /// Detection model the generated `decode_detect` stage points at.
    pub model_path: PathBuf,
    /// Hardware selector for the detect stage ("auto"/"cpu"/"cuda"/"metal").
    pub detect_hw: String,
    /// Square detector input size in pixels.
    pub detect_input_size: u32,
    /// Default detector confidence threshold (0.0–1.0).
    pub detect_conf_threshold: f32,
    /// RTSP transport for the capture stage ("tcp"/"udp").
    pub rtsp_transport: String,
    /// Optional MQTT broker URL; when set, an `output_mqtt` stage is appended.
    pub mqtt_url: Option<String>,
    /// Continuous-segment rotation length, seconds (`store` `max_secs`).
    pub segment_max_secs: u64,
    /// Event-clip pre/post-roll, seconds (`store` `pre_roll_sec`/`post_roll_sec`).
    pub pre_roll_sec: u64,
    pub post_roll_sec: u64,
    /// Event-clip ring-buffer bound, seconds (`store` `max_buffer_sec`).
    pub max_buffer_sec: u64,
    /// Trigger types the event recorder records on (`store` `trigger_types`).
    pub trigger_types: Vec<String>,
}

/// How decoded EVENTs are mapped onto Events/Frames rows.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct IngestConfig {
    /// Bounded queue between the socket readers and the ingest task. Events are
    /// dropped (with a warning) rather than stalling media when this fills.
    pub channel_capacity: usize,
    /// Name stamped on Events rows created from zm-next activity.
    pub event_name: String,
    /// Storage row id the indexed clips belong to. `None` → ZoneMinder's
    /// default (lowest-id) storage, resolved at ingest time.
    pub default_storage_id: Option<u16>,
    /// Idle gap (seconds) after which an open event with no further detections
    /// is auto-finalized if no `recording_saved` arrived. 0 disables.
    pub idle_finalize_seconds: u64,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            binary: "zm-core".to_string(),
        }
    }
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("/var/lib/zm_api/pipelines"),
            model_path: PathBuf::from("/var/lib/zm_api/models/yolo26n.onnx"),
            detect_hw: "auto".to_string(),
            detect_input_size: 640,
            detect_conf_threshold: 0.35,
            rtsp_transport: "tcp".to_string(),
            mqtt_url: None,
            segment_max_secs: 300,
            pre_roll_sec: 5,
            post_roll_sec: 10,
            max_buffer_sec: 15,
            trigger_types: vec![
                "detection".to_string(),
                "motion".to_string(),
                "audio_event".to_string(),
                "tracked_detection".to_string(),
            ],
        }
    }
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            channel_capacity: 256,
            event_name: "zm-next".to_string(),
            default_storage_id: None,
            idle_finalize_seconds: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_disabled_and_safe() {
        let cfg = ZmNextConfig::default();
        assert!(!cfg.enabled);
        assert_eq!(cfg.worker.binary, "zm-core");
        assert_eq!(cfg.ingest.channel_capacity, 256);
        assert_eq!(cfg.ingest.default_storage_id, None);
        assert_eq!(cfg.pipeline.detect_input_size, 640);
        assert!(cfg.pipeline.mqtt_url.is_none());
    }
}
