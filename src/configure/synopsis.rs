//! Configuration for the motion-synopsis optimiser + renderer + serving.
//!
//! Synopsis condenses an event (or a range of events) into a short composite
//! where many object "tubes" play at once over a clean background plate. zm-next
//! produces the ingredients (cutouts + plates) and announces them via the
//! `review_assets` (0x0306) EVENT; zm-api ingests, optimises tube placement,
//! renders, caches and serves.
//!
//! The whole section defaults to disabled: with no `[synopsis]` block the
//! ingest still records manifests (cheap), but nothing renders until enabled.

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SynopsisConfig {
    /// Master switch for rendering/serving. When false the API endpoints report
    /// the feature is disabled; ingest of manifests is unaffected.
    pub enabled: bool,
    /// Video encode+mux backend. Only `"native"` (in-process ffmpeg-next
    /// libavcodec/libavformat) is supported — zm-api never shells out to the
    /// `ffmpeg` binary in production. Any other value is rejected at render time.
    pub encoder_backend: String,
    /// Hard wall-clock cap on a single render before it is abandoned as failed.
    pub render_timeout_seconds: u64,
    /// Maximum renders running concurrently (a tokio `Semaphore`); excess
    /// requests queue.
    pub max_concurrent_renders: usize,
    /// Days a rendered artifact is retained before the cleanup job removes it.
    /// `0` disables expiry (artifacts kept until manually pruned).
    pub retention_days: u64,
    /// Directory rendered synopsis artifacts (mp4 / still) are cached in.
    pub cache_dir: PathBuf,
    /// Output synopsis frame rate for the rendered mp4 (P3).
    pub output_fps: u32,
    /// Cap on tubes composited into a single output frame; overflow extends the
    /// synopsis length rather than overcrowding. `0` = unlimited.
    pub max_tubes_per_frame: usize,
    /// Collision-area budget as a fraction of tube pixels — the single
    /// "condensation" knob (range 0.02–0.10; ~0.05 default). Higher = denser,
    /// shorter synopsis; lower = sparser, longer.
    pub collision_budget: f32,
    /// Mask edge feather radius in pixels (1–2) to suppress cutout halos.
    pub mask_feather_px: u32,
    /// Monitor ids whose zm-next pipeline should emit synopsis ingredients
    /// (polygon masks + tracker + review_export + plate_export). Empty = none.
    /// Non-synopsis cameras pay none of this cost. This policy lives in zm-api
    /// (the only DB reader), consistent with the zm-next/zm-api split.
    pub enabled_monitors: Vec<u32>,
}

impl Default for SynopsisConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            encoder_backend: "native".to_string(),
            render_timeout_seconds: 120,
            max_concurrent_renders: 2,
            retention_days: 7,
            cache_dir: PathBuf::from("/var/lib/zm_api/synopsis"),
            output_fps: 12,
            max_tubes_per_frame: 8,
            collision_budget: 0.05,
            mask_feather_px: 1,
            enabled_monitors: Vec::new(),
        }
    }
}

impl SynopsisConfig {
    /// True iff the configured encoder backend is the supported native one.
    pub fn encoder_is_native(&self) -> bool {
        self.encoder_backend.eq_ignore_ascii_case("native")
    }

    /// True iff a monitor's zm-next pipeline should emit synopsis ingredients:
    /// the feature is on globally *and* the monitor is opted in.
    pub fn pipeline_enabled_for(&self, monitor_id: u32) -> bool {
        self.enabled && self.enabled_monitors.contains(&monitor_id)
    }

    /// Clamp the collision budget into the sane 2–10% band the optimiser expects.
    pub fn clamped_collision_budget(&self) -> f32 {
        self.collision_budget.clamp(0.02, 0.10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_disabled_and_native() {
        let c = SynopsisConfig::default();
        assert!(!c.enabled);
        assert!(c.encoder_is_native());
        assert_eq!(c.retention_days, 7);
        assert_eq!(c.max_concurrent_renders, 2);
    }

    #[test]
    fn collision_budget_is_clamped() {
        let high = SynopsisConfig {
            collision_budget: 0.5,
            ..Default::default()
        };
        assert_eq!(high.clamped_collision_budget(), 0.10);
        let low = SynopsisConfig {
            collision_budget: 0.0,
            ..Default::default()
        };
        assert_eq!(low.clamped_collision_budget(), 0.02);
    }
}
