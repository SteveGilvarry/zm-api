//! Event-retention reaper.
//!
//! Periodically enforces, per `Storage` row, a free-space floor / max age /
//! byte quota by deleting whole events oldest-first — media files **and** the
//! DB rows that reference them, so the UI never shows 404 playback. This is the
//! piece ZoneMinder implements with purge filters; here it's a few numeric
//! knobs (see [`crate::configure::retention::RetentionConfig`]).
//!
//! Safety rules (never violated):
//! * `Archived` events are never deleted.
//! * In-progress events (`EndDateTime IS NULL`) are never deleted.
//! * The newest event per monitor is always kept, even if a limit is still
//!   breached — a single huge open event shouldn't be force-killed.
//!
//! DB-before-disk ordering: a crash mid-delete leaves an orphan *file* (cheap
//! to reclaim) rather than an orphan *row* (shows as broken playback).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, ModelTrait, QueryFilter,
    QueryOrder, TransactionTrait,
};
use tracing::{info, warn};

use crate::configure::retention::RetentionConfig;
use crate::entity::sea_orm_active_enums::Scheme;
use crate::entity::{
    events, events_archived, events_day, events_hour, events_month, events_week, frames, storage,
};

pub struct RetentionService {
    db: Arc<DatabaseConnection>,
    config: RetentionConfig,
}

/// Outcome of reaping a single storage, for logging.
#[derive(Default)]
struct ReapStats {
    deleted: usize,
    reclaimed: u64,
}

impl RetentionService {
    pub fn new(db: Arc<DatabaseConnection>, config: RetentionConfig) -> Self {
        Self { db, config }
    }

    /// Spawn the periodic reaper loop. Returns immediately.
    pub fn spawn(self: Arc<Self>) {
        let interval = self.config.interval();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                ticker.tick().await;
                if let Err(e) = self.reap_once().await {
                    warn!("retention pass failed: {e}");
                }
            }
        });
    }

    /// One full evaluation across every storage. Public for tests / manual runs.
    pub async fn reap_once(&self) -> Result<(), DbErr> {
        let storages = storage::Entity::find()
            .order_by_asc(storage::Column::Id)
            .all(self.db.as_ref())
            .await?;
        // Events with StorageId 0 / NULL resolve to the lowest-id storage —
        // identical to `DaemonManager::zmnext_events_root`'s fallback.
        let default_id = storages.first().map(|s| s.id);

        for st in &storages {
            let is_default = Some(st.id) == default_id;
            match self.reap_storage(st, is_default).await {
                Ok(stats) if stats.deleted > 0 => info!(
                    "retention: storage {} ({}) {} {} events / {:.2} GiB",
                    st.id,
                    st.path,
                    if self.config.dry_run {
                        "would delete"
                    } else {
                        "deleted"
                    },
                    stats.deleted,
                    stats.reclaimed as f64 / GIB,
                ),
                Ok(_) => {}
                Err(e) => warn!("retention: storage {} ({}) failed: {e}", st.id, st.path),
            }
        }
        Ok(())
    }

    async fn reap_storage(
        &self,
        st: &storage::Model,
        is_default: bool,
    ) -> Result<ReapStats, DbErr> {
        let cfg = &self.config;

        // Candidate set: events on this storage that are safe to delete.
        let mut storage_match = Condition::any().add(events::Column::StorageId.eq(st.id));
        if is_default {
            storage_match = storage_match
                .add(events::Column::StorageId.is_null())
                .add(events::Column::StorageId.eq(0));
        }
        let all: Vec<events::Model> = events::Entity::find()
            .filter(storage_match)
            .filter(events::Column::Archived.eq(0))
            .filter(events::Column::EndDateTime.is_not_null())
            .order_by_asc(events::Column::StartDateTime)
            .all(self.db.as_ref())
            .await?;

        if all.is_empty() {
            return Ok(ReapStats::default());
        }

        // Total bytes held by this storage (incl. protected events) for the quota.
        let mut used: u64 = all.iter().filter_map(|e| e.disk_space).sum();

        // Protect the newest event per monitor: since `all` is oldest-first, the
        // last id seen per monitor is its newest.
        let mut newest: HashMap<u32, u64> = HashMap::new();
        for e in &all {
            newest.insert(e.monitor_id, e.id);
        }
        let protected: HashSet<u64> = newest.into_values().collect();

        // Model free space by starting from a single statvfs and crediting each
        // deletion's bytes back — avoids a syscall per event and lets `dry_run`
        // report a realistic plan.
        let (total, mut avail) = fs_total_avail(&st.path).unwrap_or((0, 0));

        let age_cutoff = (cfg.max_age_days > 0).then(|| {
            chrono::Utc::now().naive_utc() - chrono::Duration::days(cfg.max_age_days as i64)
        });

        let mut stats = ReapStats::default();
        for ev in all {
            if protected.contains(&ev.id) {
                continue;
            }
            let free_pct = if total > 0 {
                avail as f64 / total as f64 * 100.0
            } else {
                100.0
            };
            let over_free = cfg.min_free_pct > 0.0 && free_pct < cfg.min_free_pct;
            let over_bytes = cfg.max_bytes > 0 && used > cfg.max_bytes;
            let too_old = age_cutoff
                .zip(ev.start_date_time)
                .is_some_and(|(cut, start)| start < cut);

            if !(over_free || over_bytes || too_old) {
                // Oldest-first: nothing further can be over-age, and space/quota
                // are satisfied, so we're done with this storage.
                break;
            }

            let bytes = ev.disk_space.unwrap_or(0);
            if cfg.dry_run {
                info!(
                    "retention[dry-run]: would delete event {} (monitor {}, {:.1} MiB, start {:?})",
                    ev.id,
                    ev.monitor_id,
                    bytes as f64 / MIB,
                    ev.start_date_time
                );
            } else {
                self.delete_event(&ev, st).await?;
            }
            avail = avail.saturating_add(bytes);
            used = used.saturating_sub(bytes);
            stats.deleted += 1;
            stats.reclaimed = stats.reclaimed.saturating_add(bytes);
        }

        Ok(stats)
    }

    /// Delete one event: DB rows in a transaction, then its on-disk directory.
    async fn delete_event(&self, ev: &events::Model, st: &storage::Model) -> Result<(), DbErr> {
        let dir = event_dir(st, ev);

        let txn = self.db.begin().await?;
        // No FK cascade covers these (only Snapshots_Events / Events_Tags do),
        // so delete the children explicitly before the parent.
        frames::Entity::delete_many()
            .filter(frames::Column::EventId.eq(ev.id))
            .exec(&txn)
            .await?;
        events_hour::Entity::delete_many()
            .filter(events_hour::Column::EventId.eq(ev.id))
            .exec(&txn)
            .await?;
        events_day::Entity::delete_many()
            .filter(events_day::Column::EventId.eq(ev.id))
            .exec(&txn)
            .await?;
        events_week::Entity::delete_many()
            .filter(events_week::Column::EventId.eq(ev.id))
            .exec(&txn)
            .await?;
        events_month::Entity::delete_many()
            .filter(events_month::Column::EventId.eq(ev.id))
            .exec(&txn)
            .await?;
        events_archived::Entity::delete_many()
            .filter(events_archived::Column::EventId.eq(ev.id))
            .exec(&txn)
            .await?;
        // Cascades Snapshots_Events + Events_Tags via their FKs.
        ev.clone().delete(&txn).await?;
        txn.commit().await?;

        // Files last: an orphan file is reclaimable; an orphan row is not.
        if let Some(dir) = dir {
            if dir.exists() {
                if let Err(e) = std::fs::remove_dir_all(&dir) {
                    warn!(
                        "retention: removed DB rows for event {} but failed to remove {}: {e}",
                        ev.id,
                        dir.display()
                    );
                }
            }
        }
        Ok(())
    }
}

const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
const MIB: f64 = 1024.0 * 1024.0;

/// Resolve an event's on-disk directory under its storage, honouring the
/// storage `Scheme` (zm-next writes `Medium`). Returns `None` if the resolved
/// path would escape the storage root.
fn event_dir(st: &storage::Model, ev: &events::Model) -> Option<PathBuf> {
    let root = PathBuf::from(&st.path);
    let base = root.join(ev.monitor_id.to_string());
    let dir = match (&st.scheme, ev.start_date_time) {
        (Scheme::Medium, Some(dt)) => base
            .join(dt.format("%Y-%m-%d").to_string())
            .join(ev.id.to_string()),
        (Scheme::Deep, Some(dt)) => base.join(dt.format("%y/%m/%d/%H/%M/%S").to_string()),
        // Shallow, or anything without a start time.
        _ => base.join(ev.id.to_string()),
    };
    // Defensive: never rm outside the storage root.
    if dir.starts_with(&root) {
        Some(dir)
    } else {
        None
    }
}

/// `(total_bytes, available_bytes)` for the filesystem holding `path`.
fn fs_total_avail(path: &str) -> Option<(u64, u64)> {
    let s = nix::sys::statvfs::statvfs(Path::new(path)).ok()?;
    let frag = s.fragment_size() as u64;
    Some((s.blocks() as u64 * frag, s.blocks_available() as u64 * frag))
}
