//! Configuration for the event-retention reaper (`src/service/retention`).
//!
//! Bounds recording disk usage automatically, per `Storage` location, so the
//! disk can't silently fill the way it did before this existed. Three
//! independent limits; an event is eligible for deletion if **any** is
//! exceeded, deleting oldest-first until all are satisfied. Off by default.

use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct RetentionConfig {
    /// Master switch. When false the reaper never spawns.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// How often the reaper evaluates every storage.
    #[serde(default = "default_interval_seconds")]
    pub interval_seconds: u64,

    /// Keep the filesystem holding each storage at least this percent free.
    /// The disk-full safety net. `0` disables the free-space check.
    #[serde(default = "default_min_free_pct")]
    pub min_free_pct: f64,

    /// Delete events older than this many days regardless of free space.
    /// `0` disables age-based deletion.
    #[serde(default)]
    pub max_age_days: u64,

    /// Hard cap on the bytes a single storage may hold (sum of
    /// `Events.DiskSpace`). `0` disables the quota.
    #[serde(default)]
    pub max_bytes: u64,

    /// Log what *would* be deleted each pass without deleting anything.
    #[serde(default)]
    pub dry_run: bool,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            interval_seconds: default_interval_seconds(),
            min_free_pct: default_min_free_pct(),
            max_age_days: 0,
            max_bytes: 0,
            dry_run: false,
        }
    }
}

impl RetentionConfig {
    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.interval_seconds.max(1))
    }
}

fn default_enabled() -> bool {
    false
}

fn default_interval_seconds() -> u64 {
    300
}

fn default_min_free_pct() -> f64 {
    10.0
}
