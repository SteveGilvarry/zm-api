//! Native replacements for ZoneMinder's Perl maintenance daemons.
//!
//! `zmaudit.pl`, `zmstats.pl` and `zmtelemetry.pl` are periodic housekeeping
//! jobs with no state of their own beyond the database, which makes them the
//! cheapest of the Perl daemons to absorb. Running them in-process removes
//! three supervised processes and puts their logging in the same journal as
//! everything else.
//!
//! Each is independently switchable, because their risk profiles differ
//! sharply: stats only writes rollup rows, telemetry only talks to the network,
//! and the audit deletes things.

use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct MaintenanceConfig {
    #[serde(default)]
    pub audit: AuditConfig,
    #[serde(default)]
    pub stats: StatsConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}

/// Filesystem/database consistency sweep — the `zmaudit.pl` job.
///
/// Deletes by default in the same places zmaudit does, but ships **disabled**
/// and with `dry_run` on, so enabling it without reading the log first cannot
/// remove anything. That is deliberately more cautious than the Perl, which
/// deletes as soon as it is started.
#[derive(Debug, Deserialize, Clone)]
pub struct AuditConfig {
    #[serde(default)]
    pub enabled: bool,

    /// How often to sweep.
    #[serde(default = "default_audit_interval")]
    pub interval_seconds: u64,

    /// Report what would be removed without removing it. On by default: the
    /// first run on a real install is the one most likely to reveal that an
    /// assumption about the storage layout is wrong.
    #[serde(default = "default_true")]
    pub dry_run: bool,

    /// Ignore anything younger than this. An event that is mid-recording has
    /// rows before it has files, and a directory can exist before its row is
    /// committed; without a grace period the sweep races the capture daemon and
    /// deletes live recordings.
    #[serde(default = "default_min_age_seconds")]
    pub min_age_seconds: u64,

    /// Delete `Frames` and `Stats` rows whose event no longer exists. Pure
    /// database garbage: nothing can reach them and they only grow.
    #[serde(default = "default_true")]
    pub remove_orphaned_frames: bool,

    /// Delete events that never recorded a frame and are older than
    /// `min_age_seconds`. Archived events are always skipped — zmaudit intends
    /// to skip them too but does not fetch the `Archived` column, so its guard
    /// never fires and it deletes them.
    #[serde(default = "default_true")]
    pub remove_empty_events: bool,

    /// Close events left with a NULL `EndDateTime` — a capture daemon that died
    /// mid-event. Recomputes end time, length, frame and score totals from
    /// `Frames`, and marks the event recovered. An update, never a delete.
    #[serde(default = "default_true")]
    pub close_unclosed_events: bool,

    /// Recompute `Event_Summaries` counters and `Storage.DiskSpace` from the
    /// underlying rows. Both drift: the summaries are trigger-maintained and a
    /// missed trigger is never self-corrected, and `Storage.DiskSpace` is
    /// adjusted incrementally by application code, so a crash mid-delete leaves
    /// it permanently wrong.
    #[serde(default = "default_true")]
    pub resync_counters: bool,

    /// Never delete more than this many items per check in one pass (each of
    /// the sweeps and the filesystem half has its own budget). A misconfigured
    /// storage path makes every event look orphaned; this bounds the damage to
    /// something recoverable while the log makes the cause obvious.
    #[serde(default = "default_max_deletes")]
    pub max_deletes_per_pass: usize,

    /// Reconcile event directories on disk against `Events` rows.
    ///
    /// Off even when the audit is on, because it is the only part that touches
    /// the filesystem. Orphans are moved to a quarantine directory rather than
    /// deleted, so enabling it is recoverable.
    #[serde(default)]
    pub filesystem: FilesystemAuditConfig,
}

/// The filesystem half of the audit.
///
/// Deliberately does **not** work the way `zmaudit.pl` does. zmaudit computes an
/// event's directory from its `StartDateTime` and `rm -rf`s the result; that
/// path can be wrong from a timezone mismatch, a corrupted timestamp, a `Scheme`
/// changed after recording, or a wrong `StorageId`, and when it is wrong the
/// thing removed is an unrelated directory.
///
/// Here nothing destructive ever acts on a computed path. Directories are
/// discovered by walking, identified from evidence inside them, and only a path
/// that was actually enumerated is ever moved.
#[derive(Debug, Deserialize, Clone)]
pub struct FilesystemAuditConfig {
    #[serde(default)]
    pub enabled: bool,

    /// Move directories with no `Events` row into quarantine.
    #[serde(default = "default_true")]
    pub quarantine_orphaned_dirs: bool,

    /// Delete `Events` rows whose directory was not found on disk.
    ///
    /// Independent of the above and more dangerous, because the evidence is an
    /// absence: a storage that failed to mount looks exactly like every event
    /// being gone. A pass refuses this half when more rows are missing than
    /// events were found on disk, which is what that looks like — but leave
    /// this off unless the broken-playback rows are actually a problem.
    #[serde(default)]
    pub remove_rows_without_media: bool,

    /// Where quarantined directories are moved, relative to each storage root.
    /// A rename within one filesystem, so it is atomic and costs nothing.
    #[serde(default = "default_quarantine_dir")]
    pub quarantine_dir: String,

    /// Delete quarantined directories after this many days. `0` keeps them
    /// forever, which fills the disk — the thing the audit exists to prevent.
    #[serde(default = "default_quarantine_days")]
    pub quarantine_retention_days: u64,

    /// Require an orphan to be seen in this many consecutive passes before
    /// acting. Two means a row committed after the walk started, or a directory
    /// created just before its row, is never mistaken for an orphan.
    #[serde(default = "default_confirmations")]
    pub confirmations_required: u32,

    /// Maximum directory depth to descend below a monitor directory. The Deep
    /// scheme needs six (`yy/mm/dd/HH/MM/SS`); anything deeper is not a layout
    /// ZoneMinder produces.
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
}

fn default_quarantine_dir() -> String {
    ".zm-api-quarantine".to_string()
}
fn default_quarantine_days() -> u64 {
    7
}
fn default_confirmations() -> u32 {
    2
}
fn default_max_depth() -> usize {
    6
}

impl Default for FilesystemAuditConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            quarantine_orphaned_dirs: default_true(),
            remove_rows_without_media: false,
            quarantine_dir: default_quarantine_dir(),
            quarantine_retention_days: default_quarantine_days(),
            confirmations_required: default_confirmations(),
            max_depth: default_max_depth(),
        }
    }
}

/// Rolling-window rollup maintenance — the `zmstats.pl` job.
///
/// `Events_Hour/Day/Week/Month` hold one row per event inside each window.
/// Database triggers keep `Event_Summaries` counters in step with those rows,
/// so the only work is adding events that have entered a window and removing
/// those that have aged out — the counters then follow automatically.
#[derive(Debug, Deserialize, Clone)]
pub struct StatsConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_stats_interval")]
    pub interval_seconds: u64,
}

/// Anonymous usage reporting — the `zmtelemetry.pl` job.
///
/// Off by default and stays off unless explicitly enabled. A phone-home that
/// switches itself on is a poor default regardless of how little it sends.
#[derive(Debug, Deserialize, Clone)]
pub struct TelemetryConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default = "default_telemetry_interval")]
    pub interval_seconds: u64,

    /// Where the report is sent.
    #[serde(default = "default_telemetry_endpoint")]
    pub endpoint: String,
}

fn default_true() -> bool {
    true
}
fn default_audit_interval() -> u64 {
    3600
}
fn default_min_age_seconds() -> u64 {
    3600
}
fn default_max_deletes() -> usize {
    1000
}
fn default_stats_interval() -> u64 {
    300
}
fn default_telemetry_interval() -> u64 {
    86400
}
fn default_telemetry_endpoint() -> String {
    "https://telemetry.zoneminder.com/index.php".to_string()
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_seconds: default_audit_interval(),
            dry_run: default_true(),
            min_age_seconds: default_min_age_seconds(),
            remove_orphaned_frames: default_true(),
            remove_empty_events: default_true(),
            close_unclosed_events: default_true(),
            resync_counters: default_true(),
            filesystem: FilesystemAuditConfig::default(),
            max_deletes_per_pass: default_max_deletes(),
        }
    }
}

impl Default for StatsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_seconds: default_stats_interval(),
        }
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_seconds: default_telemetry_interval(),
            endpoint: default_telemetry_endpoint(),
        }
    }
}

impl AuditConfig {
    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.interval_seconds.max(60))
    }
    pub fn min_age(&self) -> Duration {
        Duration::from_secs(self.min_age_seconds)
    }
}

impl StatsConfig {
    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.interval_seconds.max(30))
    }
}

impl TelemetryConfig {
    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.interval_seconds.max(3600))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_is_off_by_default() {
        let cfg = MaintenanceConfig::default();
        assert!(!cfg.audit.enabled);
        assert!(!cfg.stats.enabled);
        assert!(
            !cfg.telemetry.enabled,
            "telemetry must never enable itself: it leaves the machine"
        );
    }

    #[test]
    fn the_audit_is_dry_by_default() {
        // Enabling the audit must not delete anything until the operator has
        // read a pass and turned dry_run off deliberately.
        assert!(AuditConfig::default().dry_run);
    }

    #[test]
    fn intervals_have_a_sane_floor() {
        // A zero or tiny interval in config would busy-loop against the
        // database; clamp rather than trust the file.
        let audit = AuditConfig {
            interval_seconds: 0,
            ..AuditConfig::default()
        };
        assert_eq!(audit.interval(), Duration::from_secs(60));

        let telemetry = TelemetryConfig {
            interval_seconds: 1,
            ..TelemetryConfig::default()
        };
        assert_eq!(telemetry.interval(), Duration::from_secs(3600));
    }
}
