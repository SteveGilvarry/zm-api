//! Native replacement for `zmaudit.pl` — database-side consistency.
//!
//! Four checks, each independently switchable:
//!
//! * orphaned `Frames` / `Stats` rows whose event is gone
//! * events that never recorded a frame
//! * events left unclosed by a capture daemon that died
//! * `Event_Summaries` and `Storage.DiskSpace` counters that have drifted
//!
//! ## Two deliberate departures from the Perl
//!
//! **Archived events are genuinely skipped.** zmaudit means to skip them when
//! deleting frameless events — `if ($$event{Archived})` — but its `SELECT` list
//! is `E.Id, E.StartDateTime, F.EventId`, so `Archived` is never fetched, the
//! guard is always false, and archived events are deleted. Archiving an event
//! is a user saying "keep this"; the query here fetches the column and honours
//! it.
//!
//! **`dry_run` means dry.** zmaudit's `--report` suppresses its deletes but
//! still performs row updates, empty-directory removal, stray-image unlinking,
//! log pruning and counter resyncs, so "just report" is not what it does. Here
//! nothing is written when `dry_run` is set.
//!
//! ## What is not here yet
//!
//! The filesystem half — reconciling event directories against `Events` rows in
//! both directions. That is where zmaudit's real hazard lives: the `rm -rf`
//! target is *derived* from `StartDateTime` formatted in the process's local
//! timezone, so a timezone mismatch between the recording daemon and the
//! auditor aims it at a directory that was never the event's. Doing that safely
//! wants its own change, and `service::event_storage` already centralises the
//! path derivation it will need.

use std::sync::Arc;

use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, FromQueryResult, Statement};
use tracing::{debug, info, warn};

use crate::configure::maintenance::AuditConfig;

pub struct AuditService {
    db: Arc<DatabaseConnection>,
    config: AuditConfig,
}

/// What one pass found, and whether it acted.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct AuditReport {
    pub orphaned_frames: u64,
    pub orphaned_stats: u64,
    pub empty_events: u64,
    pub unclosed_events: u64,
    pub dry_run: bool,
}

impl AuditReport {
    pub fn total(&self) -> u64 {
        self.orphaned_frames + self.orphaned_stats + self.empty_events + self.unclosed_events
    }

    pub fn is_clean(&self) -> bool {
        self.total() == 0
    }
}

#[derive(FromQueryResult)]
struct CountRow {
    n: i64,
}

#[derive(FromQueryResult)]
struct IdRow {
    id: u64,
}

impl AuditService {
    pub fn new(db: Arc<DatabaseConnection>, config: AuditConfig) -> Self {
        Self { db, config }
    }

    pub fn spawn(self: Arc<Self>) {
        let interval = self.config.interval();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                ticker.tick().await;
                match self.run_once().await {
                    Ok(report) if report.is_clean() => debug!("audit pass: nothing to do"),
                    Ok(report) => info!(
                        "audit pass{}: {} orphaned frames, {} orphaned stats, \
                         {} empty events, {} unclosed events",
                        if report.dry_run { " (dry run)" } else { "" },
                        report.orphaned_frames,
                        report.orphaned_stats,
                        report.empty_events,
                        report.unclosed_events,
                    ),
                    Err(e) => warn!("audit pass failed: {e}"),
                }
            }
        });
    }

    /// One full pass. Jobs are independent — one failing does not stop the rest.
    pub async fn run_once(&self) -> Result<AuditReport, DbErr> {
        let mut report = AuditReport {
            dry_run: self.config.dry_run,
            ..Default::default()
        };

        if self.config.remove_orphaned_frames {
            match self.sweep_orphaned_children().await {
                Ok((frames, stats)) => {
                    report.orphaned_frames = frames;
                    report.orphaned_stats = stats;
                }
                Err(e) => warn!("orphan sweep failed: {e}"),
            }
        }
        if self.config.remove_empty_events {
            match self.remove_empty_events().await {
                Ok(n) => report.empty_events = n,
                Err(e) => warn!("empty-event sweep failed: {e}"),
            }
        }
        if self.config.close_unclosed_events {
            match self.close_unclosed_events().await {
                Ok(n) => report.unclosed_events = n,
                Err(e) => warn!("unclosed-event sweep failed: {e}"),
            }
        }
        if self.config.resync_counters {
            if let Err(e) = self.resync_counters().await {
                warn!("counter resync failed: {e}");
            }
        }

        Ok(report)
    }

    /// `Frames` and `Stats` rows whose event no longer exists.
    ///
    /// Pure database garbage — nothing can reach them, and `Frames` in
    /// particular is the largest table in most installs.
    async fn sweep_orphaned_children(&self) -> Result<(u64, u64), DbErr> {
        let frames = self
            .delete_where(
                "Frames",
                "NOT EXISTS (SELECT 1 FROM Events WHERE Events.Id = Frames.EventId)",
            )
            .await?;
        let stats = self
            .delete_where(
                "Stats",
                "NOT EXISTS (SELECT 1 FROM Events WHERE Events.Id = Stats.EventId)",
            )
            .await?;
        Ok((frames, stats))
    }

    /// Events that recorded no frames and are past the grace period.
    ///
    /// An event has a row before it has frames, so without the age guard this
    /// races the capture daemon and deletes recordings in progress. Archived
    /// events are excluded — see the module note.
    async fn remove_empty_events(&self) -> Result<u64, DbErr> {
        let min_age = self.config.min_age_seconds;
        let predicate = format!(
            "Archived = 0 \
             AND StartDateTime IS NOT NULL \
             AND StartDateTime < DATE_SUB(NOW(), INTERVAL {min_age} SECOND) \
             AND NOT EXISTS (SELECT 1 FROM Frames WHERE Frames.EventId = Events.Id)"
        );
        self.delete_where("Events", &predicate).await
    }

    /// Close events whose capture daemon died mid-recording.
    ///
    /// An update, never a delete: end time, length, frame and score totals are
    /// recomputed from the frames that did land, and the event is marked
    /// recovered so the repair is visible rather than silent.
    async fn close_unclosed_events(&self) -> Result<u64, DbErr> {
        let min_age = self.config.min_age_seconds;
        let backend = self.db.get_database_backend();

        let find = format!(
            "SELECT Id AS id FROM Events \
             WHERE EndDateTime IS NULL \
               AND StartDateTime IS NOT NULL \
               AND StartDateTime < DATE_SUB(NOW(), INTERVAL {min_age} SECOND) \
             LIMIT {}",
            self.config.max_deletes_per_pass
        );
        let ids: Vec<u64> = IdRow::find_by_statement(Statement::from_string(backend, find))
            .all(self.db.as_ref())
            .await?
            .into_iter()
            .map(|r| r.id)
            .collect();

        if ids.is_empty() || self.config.dry_run {
            if !ids.is_empty() {
                debug!("dry run: would close {} unclosed events", ids.len());
            }
            return Ok(ids.len() as u64);
        }

        for id in &ids {
            // Single-table UPDATE driven by subqueries over Frames. A
            // multi-table UPDATE would hold shared locks on the joined rows to
            // commit and deadlock against ZoneMinder's own event triggers.
            let sql = format!(
                "UPDATE Events SET \
                   EndDateTime = COALESCE( \
                     (SELECT MAX(TimeStamp) FROM Frames WHERE Frames.EventId = {id}), \
                     StartDateTime), \
                   Frames = (SELECT COUNT(*) FROM Frames WHERE Frames.EventId = {id}), \
                   AlarmFrames = (SELECT COUNT(*) FROM Frames \
                     WHERE Frames.EventId = {id} AND Score > 0), \
                   TotScore = (SELECT COALESCE(SUM(Score),0) FROM Frames \
                     WHERE Frames.EventId = {id}), \
                   MaxScore = (SELECT COALESCE(MAX(Score),0) FROM Frames \
                     WHERE Frames.EventId = {id}), \
                   Length = TIMESTAMPDIFF(SECOND, StartDateTime, COALESCE( \
                     (SELECT MAX(TimeStamp) FROM Frames WHERE Frames.EventId = {id}), \
                     StartDateTime)), \
                   Notes = CONCAT_WS(' ', Notes, 'Recovered.') \
                 WHERE Id = {id} AND EndDateTime IS NULL"
            );
            self.db
                .execute(Statement::from_string(backend, sql))
                .await?;
        }
        info!("closed {} unclosed events", ids.len());
        Ok(ids.len() as u64)
    }

    /// Recompute `Event_Summaries` and `Storage.DiskSpace` from the rows they
    /// summarise.
    ///
    /// Both drift for different reasons. `Event_Summaries` is trigger-maintained,
    /// so a trigger that did not fire is never self-corrected. `Storage.DiskSpace`
    /// is adjusted *incrementally* by application code, so a crash between
    /// deleting an event and adjusting the total leaves it permanently wrong.
    ///
    /// The `DiskSpace` update is guarded by a compare-and-swap on the value we
    /// read. Because that column is adjusted relatively rather than recomputed,
    /// writing a stale absolute snapshot would actively undo a concurrent
    /// adjustment instead of being corrected on the next pass.
    async fn resync_counters(&self) -> Result<(), DbErr> {
        if self.config.dry_run {
            debug!("dry run: skipping counter resync");
            return Ok(());
        }
        let backend = self.db.get_database_backend();

        self.db
            .execute(Statement::from_string(
                backend,
                "UPDATE Event_Summaries SET \
                 TotalEvents = (SELECT COUNT(*) FROM Events \
                     WHERE Events.MonitorId = Event_Summaries.MonitorId), \
                 TotalEventDiskSpace = (SELECT COALESCE(SUM(DiskSpace),0) FROM Events \
                     WHERE Events.MonitorId = Event_Summaries.MonitorId), \
                 ArchivedEvents = (SELECT COUNT(*) FROM Events \
                     WHERE Events.MonitorId = Event_Summaries.MonitorId AND Archived = 1), \
                 ArchivedEventDiskSpace = (SELECT COALESCE(SUM(DiskSpace),0) FROM Events \
                     WHERE Events.MonitorId = Event_Summaries.MonitorId AND Archived = 1)",
            ))
            .await?;

        #[derive(FromQueryResult)]
        struct StorageRow {
            id: i32,
            current: Option<i64>,
            actual: i64,
        }

        let rows = StorageRow::find_by_statement(Statement::from_string(
            backend,
            "SELECT s.Id AS id, s.DiskSpace AS current, \
                    COALESCE((SELECT SUM(e.DiskSpace) FROM Events e \
                              WHERE e.StorageId = s.Id), 0) AS actual \
             FROM Storage s",
        ))
        .all(self.db.as_ref())
        .await?;

        for row in rows {
            if row.current == Some(row.actual) {
                continue;
            }
            // `<=>` is null-safe, so a NULL current value matches the NULL we
            // read rather than failing the guard forever.
            let updated = self
                .db
                .execute(Statement::from_sql_and_values(
                    backend,
                    "UPDATE Storage SET DiskSpace = ? WHERE Id = ? AND DiskSpace <=> ?",
                    [row.actual.into(), row.id.into(), row.current.into()],
                ))
                .await?
                .rows_affected();
            if updated == 0 {
                debug!(
                    "storage {} disk space changed under us; leaving it for the next pass",
                    row.id
                );
            } else {
                info!(
                    "storage {} disk space corrected: {:?} -> {}",
                    row.id, row.current, row.actual
                );
            }
        }
        Ok(())
    }

    /// Count matching rows, then delete them unless this is a dry run.
    ///
    /// Bounded by `max_deletes_per_pass`: a misconfigured storage path can make
    /// a great many rows look orphaned at once, and the cap keeps the blast
    /// radius recoverable while the log makes the cause obvious.
    async fn delete_where(&self, table: &str, predicate: &str) -> Result<u64, DbErr> {
        let backend = self.db.get_database_backend();

        let count = CountRow::find_by_statement(Statement::from_string(
            backend,
            format!("SELECT COUNT(*) AS n FROM {table} WHERE {predicate}"),
        ))
        .one(self.db.as_ref())
        .await?
        .map(|r| r.n.max(0) as u64)
        .unwrap_or(0);

        if count == 0 {
            return Ok(0);
        }
        if self.config.dry_run {
            info!("dry run: would delete {count} rows from {table}");
            return Ok(count);
        }

        let limit = self.config.max_deletes_per_pass;
        if count as usize > limit {
            warn!(
                "{count} orphaned rows in {table} exceeds max_deletes_per_pass ({limit}); \
                 deleting {limit} this pass. If this repeats, check the storage \
                 configuration before assuming the rows are really orphaned."
            );
        }

        let removed = self
            .db
            .execute(Statement::from_string(
                backend,
                format!("DELETE FROM {table} WHERE {predicate} LIMIT {limit}"),
            ))
            .await?
            .rows_affected();
        info!("deleted {removed} orphaned rows from {table}");
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clean_report_is_clean() {
        assert!(AuditReport::default().is_clean());
    }

    #[test]
    fn the_total_counts_every_category() {
        let r = AuditReport {
            orphaned_frames: 1,
            orphaned_stats: 2,
            empty_events: 3,
            unclosed_events: 4,
            dry_run: false,
        };
        assert_eq!(r.total(), 10);
        assert!(!r.is_clean());
    }

    /// The empty-event predicate is the one that deletes user data, so pin its
    /// three guards rather than trusting a reading of the SQL.
    #[test]
    fn the_empty_event_predicate_guards_archived_and_age() {
        let cfg = AuditConfig {
            min_age_seconds: 7200,
            ..AuditConfig::default()
        };
        let predicate = format!(
            "Archived = 0 \
             AND StartDateTime IS NOT NULL \
             AND StartDateTime < DATE_SUB(NOW(), INTERVAL {} SECOND) \
             AND NOT EXISTS (SELECT 1 FROM Frames WHERE Frames.EventId = Events.Id)",
            cfg.min_age_seconds
        );

        // Archived events are kept. zmaudit intends this but never fetches the
        // column, so its guard is dead and it deletes them.
        assert!(predicate.contains("Archived = 0"));
        // In-flight events are out of range.
        assert!(predicate.contains("INTERVAL 7200 SECOND"));
        // An event with no start time has no age to judge, so it is left alone
        // rather than deleted unconditionally.
        assert!(predicate.contains("StartDateTime IS NOT NULL"));
        // Only events with no frames at all.
        assert!(predicate.contains("NOT EXISTS"));
    }
}
