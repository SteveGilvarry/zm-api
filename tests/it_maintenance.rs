//! Integration tests for the native replacements of `zmaudit.pl` and
//! `zmstats.pl`, against a real schema.
//!
//! The unit tests cover the parsers and the arithmetic; these cover the SQL,
//! which is where a reimplementation of a destructive daemon actually goes
//! wrong. Every deleting path is checked twice: that it removes what it should,
//! and that it leaves alone what it must.
//!
//! Requires the test database — run with:
//!   APP_PROFILE=test-db cargo test --test it_maintenance -- --include-ignored

mod common;

use std::sync::Arc;

use common::test_db::get_test_db;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use zm_api::configure::maintenance::{AuditConfig, StatsConfig};
use zm_api::service::maintenance::{audit::AuditService, stats::StatsService};

/// Monitor ids well clear of anything a fixture or a real install would use.
///
/// One per test, because these run in parallel against a shared database and
/// each test's cleanup deletes by monitor id — a single shared id has them
/// deleting each other's fixtures mid-run.
const MON_EMPTY: u32 = 99_311;
const MON_FRAMES: u32 = 99_312;
const MON_UNCLOSED: u32 = 99_313;
const MON_COUNTERS: u32 = 99_314;

async fn exec(db: &DatabaseConnection, sql: impl Into<String>) {
    db.execute(Statement::from_string(
        db.get_database_backend(),
        sql.into(),
    ))
    .await
    .expect("statement");
}

async fn scalar(db: &DatabaseConnection, sql: &str) -> i64 {
    use sea_orm::FromQueryResult;
    #[derive(FromQueryResult)]
    struct Row {
        n: i64,
    }
    Row::find_by_statement(Statement::from_string(
        db.get_database_backend(),
        sql.to_string(),
    ))
    .one(db)
    .await
    .expect("query")
    .map(|r| r.n)
    .unwrap_or(0)
}

/// Remove everything this test file creates, so a failed run cannot poison the
/// next one.
async fn cleanup(db: &DatabaseConnection, monitor: u32, first_event: u64, last_event: u64) {
    exec(
        db,
        format!("DELETE FROM Frames WHERE EventId BETWEEN {first_event} AND {last_event}"),
    )
    .await;
    exec(
        db,
        format!("DELETE FROM Events WHERE MonitorId = {monitor}"),
    )
    .await;
    exec(
        db,
        format!("DELETE FROM Event_Summaries WHERE MonitorId = {monitor}"),
    )
    .await;
}

fn audit_config(dry_run: bool) -> AuditConfig {
    AuditConfig {
        enabled: true,
        dry_run,
        min_age_seconds: 3600,
        ..AuditConfig::default()
    }
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn empty_events_are_deleted_but_only_when_old_and_unarchived() {
    const MON: u32 = MON_EMPTY;
    let db = Arc::new(get_test_db().await.expect("test db"));
    cleanup(&db, MON, 9_930_001, 9_930_004).await;

    // 1: old, no frames, not archived  -> must be deleted
    // 2: old, no frames, ARCHIVED      -> must survive (zmaudit deletes this;
    //                                     its Archived guard never fires
    //                                     because the column is not selected)
    // 3: recent, no frames             -> must survive (still recording)
    // 4: old, has a frame              -> must survive
    for (id, age_secs, archived) in [
        (9_930_001u64, 7200, 0),
        (9_930_002, 7200, 1),
        (9_930_003, 60, 0),
        (9_930_004, 7200, 0),
    ] {
        exec(
            &db,
            format!(
                "INSERT INTO Events (Id, MonitorId, StateId, StartDateTime, Archived, Scheme) \
                 VALUES ({id}, {MON}, 1, DATE_SUB(NOW(), INTERVAL {age_secs} SECOND), \
                 {archived}, 'Deep')"
            ),
        )
        .await;
    }
    exec(
        &db,
        "INSERT INTO Frames (EventId, FrameId, Type, TimeStamp, Delta, Score) \
         VALUES (9930004, 1, 'Normal', NOW(), 0, 0)",
    )
    .await;

    // A dry run must change nothing while still reporting the finding.
    let dry = AuditService::new(Arc::clone(&db), audit_config(true));
    let report = dry.run_once().await.expect("dry pass");
    assert_eq!(report.empty_events, 1, "dry run should find exactly one");
    assert_eq!(
        scalar(
            &db,
            &format!("SELECT COUNT(*) AS n FROM Events WHERE MonitorId={MON}")
        )
        .await,
        4,
        "dry run must not delete anything"
    );

    let live = AuditService::new(Arc::clone(&db), audit_config(false));
    live.run_once().await.expect("live pass");

    let survivors = scalar(
        &db,
        &format!("SELECT COUNT(*) AS n FROM Events WHERE MonitorId={MON}"),
    )
    .await;
    assert_eq!(survivors, 3, "only the old unarchived frameless event goes");
    assert_eq!(
        scalar(&db, "SELECT COUNT(*) AS n FROM Events WHERE Id=9930001").await,
        0
    );
    assert_eq!(
        scalar(&db, "SELECT COUNT(*) AS n FROM Events WHERE Id=9930002").await,
        1,
        "an archived event must never be deleted for being empty"
    );
    assert_eq!(
        scalar(&db, "SELECT COUNT(*) AS n FROM Events WHERE Id=9930003").await,
        1,
        "an event younger than min_age is still recording"
    );

    cleanup(&db, MON, 9_930_001, 9_930_004).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn orphaned_frames_are_removed_and_live_ones_kept() {
    const MON: u32 = MON_FRAMES;
    let db = Arc::new(get_test_db().await.expect("test db"));
    cleanup(&db, MON, 9_930_010, 9_930_011).await;

    exec(
        &db,
        format!(
            "INSERT INTO Events (Id, MonitorId, StateId, StartDateTime, Scheme) \
             VALUES (9930010, {MON}, 1, NOW(), 'Deep')"
        ),
    )
    .await;
    // One frame on a real event, one pointing at an event that never existed.
    exec(
        &db,
        "INSERT INTO Frames (EventId, FrameId, Type, TimeStamp, Delta, Score) \
         VALUES (9930010, 1, 'Normal', NOW(), 0, 0), \
                (9930011, 1, 'Normal', NOW(), 0, 0)",
    )
    .await;

    let audit = AuditService::new(Arc::clone(&db), audit_config(false));
    audit.run_once().await.expect("pass");

    assert_eq!(
        scalar(
            &db,
            "SELECT COUNT(*) AS n FROM Frames WHERE EventId=9930011"
        )
        .await,
        0,
        "the orphan should be gone"
    );
    assert_eq!(
        scalar(
            &db,
            "SELECT COUNT(*) AS n FROM Frames WHERE EventId=9930010"
        )
        .await,
        1,
        "a frame belonging to a live event must be kept"
    );

    cleanup(&db, MON, 9_930_010, 9_930_011).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn an_unclosed_event_is_closed_from_its_frames() {
    const MON: u32 = MON_UNCLOSED;
    let db = Arc::new(get_test_db().await.expect("test db"));
    cleanup(&db, MON, 9_930_020, 9_930_020).await;

    exec(
        &db,
        format!(
            "INSERT INTO Events (Id, MonitorId, StateId, StartDateTime, EndDateTime, Scheme, Notes) \
             VALUES (9930020, {MON}, 1, DATE_SUB(NOW(), INTERVAL 2 HOUR), NULL, 'Deep', '')"
        ),
    )
    .await;
    exec(
        &db,
        "INSERT INTO Frames (EventId, FrameId, Type, TimeStamp, Delta, Score) VALUES \
         (9930020, 1, 'Normal', DATE_SUB(NOW(), INTERVAL 2 HOUR), 0, 0), \
         (9930020, 2, 'Alarm',  DATE_SUB(NOW(), INTERVAL 119 MINUTE), 0, 7), \
         (9930020, 3, 'Alarm',  DATE_SUB(NOW(), INTERVAL 118 MINUTE), 0, 3)",
    )
    .await;

    let audit = AuditService::new(Arc::clone(&db), audit_config(false));
    audit.run_once().await.expect("pass");

    use sea_orm::FromQueryResult;
    // Column types are the schema's, not conveniently uniform: Frames and
    // AlarmFrames are INT UNSIGNED, the scores are smaller still. Cast in SQL
    // so the decode does not depend on remembering each width.
    #[derive(FromQueryResult)]
    struct Closed {
        frames: i64,
        alarm_frames: i64,
        tot_score: i64,
        max_score: i64,
        still_open: i64,
        notes: String,
    }
    let row = Closed::find_by_statement(Statement::from_string(
        db.get_database_backend(),
        "SELECT CAST(Frames AS SIGNED) AS frames, \
                CAST(AlarmFrames AS SIGNED) AS alarm_frames, \
                CAST(TotScore AS SIGNED) AS tot_score, \
                CAST(MaxScore AS SIGNED) AS max_score, \
                CAST(ISNULL(EndDateTime) AS SIGNED) AS still_open, \
                Notes AS notes \
         FROM Events WHERE Id = 9930020"
            .to_string(),
    ))
    .one(db.as_ref())
    .await
    .expect("query")
    .expect("event still present");

    assert_eq!(row.still_open, 0, "EndDateTime should now be set");
    assert_eq!(row.frames, 3);
    assert_eq!(row.alarm_frames, 2, "only frames with a score above zero");
    assert_eq!(row.tot_score, 10);
    assert_eq!(row.max_score, 7);
    assert!(
        row.notes.contains("Recovered."),
        "the repair should be visible, got {:?}",
        row.notes
    );

    cleanup(&db, MON, 9_930_020, 9_930_020).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn counter_resync_corrects_drift() {
    const MON: u32 = MON_COUNTERS;
    let db = Arc::new(get_test_db().await.expect("test db"));
    cleanup(&db, MON, 9_930_030, 9_930_031).await;

    exec(
        &db,
        format!(
            "INSERT INTO Events (Id, MonitorId, StateId, StartDateTime, DiskSpace, Archived, Scheme) \
             VALUES (9930030, {MON}, 1, NOW(), 1000, 0, 'Deep'), \
                    (9930031, {MON}, 1, NOW(), 2000, 1, 'Deep')"
        ),
    )
    .await;
    // Deliberately wrong counters, as a missed trigger would leave them.
    exec(
        &db,
        format!(
            "REPLACE INTO Event_Summaries \
             (MonitorId, TotalEvents, TotalEventDiskSpace, ArchivedEvents, ArchivedEventDiskSpace) \
             VALUES ({MON}, 99, 99999, 99, 99999)"
        ),
    )
    .await;

    let audit = AuditService::new(Arc::clone(&db), audit_config(false));
    audit.run_once().await.expect("pass");

    let total = scalar(
        &db,
        &format!("SELECT TotalEvents AS n FROM Event_Summaries WHERE MonitorId={MON}"),
    )
    .await;
    let disk = scalar(
        &db,
        &format!("SELECT TotalEventDiskSpace AS n FROM Event_Summaries WHERE MonitorId={MON}"),
    )
    .await;
    let archived = scalar(
        &db,
        &format!("SELECT ArchivedEvents AS n FROM Event_Summaries WHERE MonitorId={MON}"),
    )
    .await;

    assert_eq!(total, 2, "TotalEvents should be recomputed from Events");
    assert_eq!(disk, 3000);
    assert_eq!(archived, 1);

    cleanup(&db, MON, 9_930_030, 9_930_031).await;
}

#[tokio::test]
#[ignore = "requires the test database (APP_PROFILE=test-db)"]
async fn a_stats_pass_runs_clean_against_a_real_schema() {
    // Every statement zmstats issues, executed once. This does not assert on
    // counts — the shared test database has other tests' rows in it — but it
    // does prove the SQL is valid against the real schema, which is the failure
    // mode that matters for hand-written statements.
    let db = Arc::new(get_test_db().await.expect("test db"));
    let stats = StatsService::new(
        Arc::clone(&db),
        StatsConfig {
            enabled: true,
            interval_seconds: 300,
        },
    );
    stats.run_once().await.expect("a stats pass must not error");
    // Twice, so the CPU-delta branch runs with a primed baseline too.
    stats.run_once().await.expect("second stats pass");
}
