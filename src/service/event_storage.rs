//! Event on-disk storage location and media lifecycle.
//!
//! ZoneMinder writes each event's media (recorded video, per-frame JPEGs, the
//! HLS index, etc.) into a directory under a `Storages.Path` root, laid out by
//! the storage *scheme*. This module is the single source of truth for
//! computing that directory, shared by playback (which serves the files) and
//! deletion (which removes them) so the two can never disagree about where an
//! event lives — a disagreement would mean either serving the wrong files or
//! deleting the wrong directory.

use std::io::ErrorKind;
use std::path::PathBuf;

use chrono::{Datelike, Timelike};
use tracing::{debug, info, warn};

use crate::entity::events::Model as EventModel;
use crate::entity::sea_orm_active_enums::Scheme;
use crate::error::{AppError, AppResult};
use crate::repo;
use crate::server::state::AppState;

/// Resolve the storage path for an event, falling back to the config default
/// when the event's storage row is missing, and rejecting any value that
/// contains a `..` traversal component.
///
/// `Storages.Path` is only writable by an admin (`System:Edit`), but the API
/// runs as `www-data` and serves/removes files based on that path — a
/// malicious or mistaken admin entry like `/var/cache/zm/events/..` could let
/// an otherwise-permitted operation resolve outside the events tree. Reject
/// those at the DB-read boundary.
pub(crate) async fn resolve_event_storage_path(
    state: &AppState,
    event: &EventModel,
) -> AppResult<String> {
    let storage_path = if is_default_storage(event.storage_id) {
        default_storage_path(state).await?
    } else {
        let sid = event.storage_id.expect("non-default storage_id is Some");
        match repo::storage::find_by_id(state.db(), sid).await? {
            Some(s) => s.path,
            None => {
                warn!(
                    "Storage {} not found in database, using default storage",
                    sid
                );
                default_storage_path(state).await?
            }
        }
    };

    if crate::util::path::contains_traversal(&storage_path) {
        warn!(
            "Refusing storage path with '..' traversal: {:?}",
            storage_path
        );
        return Err(AppError::InternalServerError(
            "storage path contains '..' traversal".to_string(),
        ));
    }

    Ok(storage_path)
}

/// True iff `storage_id` denotes ZoneMinder's primary/default storage rather
/// than a real `Storage` row. ZoneMinder writes `Events.StorageId = 0` (and
/// occasionally NULL) as a sentinel meaning "the default storage" — it is not a
/// foreign key, so looking up row 0 always misses.
pub(crate) fn is_default_storage(storage_id: Option<u16>) -> bool {
    matches!(storage_id, None | Some(0))
}

/// Resolve the default events directory: ZoneMinder's primary storage row when
/// the `Storage` table is populated, otherwise the configured `events_dir`.
pub(crate) async fn default_storage_path(state: &AppState) -> AppResult<String> {
    if let Some(s) = repo::storage::find_default(state.db()).await? {
        return Ok(s.path);
    }
    Ok(state.config.streaming.zoneminder.events_dir.clone())
}

/// Build the event directory path based on ZoneMinder's storage scheme.
///
/// The returned path is the event's **own leaf directory** in every scheme, so
/// it holds only that event's media (removing it does not touch siblings):
/// - **Deep**: `{storage}/{MonitorId}/{YY/MM/DD/HH/MM/SS}/`
/// - **Medium**: `{storage}/{MonitorId}/{YYYY-MM-DD}/{EventId}/`
/// - **Shallow**: `{storage}/{MonitorId}/{EventId}/`
pub(crate) fn build_event_directory_path(
    storage_path: &str,
    monitor_id: u32,
    event_id: u64,
    start_time: Option<chrono::NaiveDateTime>,
    scheme: &Scheme,
) -> PathBuf {
    let base = PathBuf::from(storage_path).join(monitor_id.to_string());

    match scheme {
        Scheme::Deep => {
            // Deep: {storage}/{monitor_id}/{YY/MM/DD/HH/MM/SS}/
            if let Some(dt) = start_time {
                base.join(format!("{:02}", dt.year() % 100))
                    .join(format!("{:02}", dt.month()))
                    .join(format!("{:02}", dt.day()))
                    .join(format!("{:02}", dt.hour()))
                    .join(format!("{:02}", dt.minute()))
                    .join(format!("{:02}", dt.second()))
            } else {
                // Fallback to shallow if no start time
                base.join(event_id.to_string())
            }
        }
        Scheme::Medium => {
            // Medium: {storage}/{monitor_id}/{YYYY-MM-DD}/{event_id}/
            if let Some(dt) = start_time {
                base.join(format!(
                    "{:04}-{:02}-{:02}",
                    dt.year(),
                    dt.month(),
                    dt.day()
                ))
                .join(event_id.to_string())
            } else {
                // Fallback to shallow if no start time
                base.join(event_id.to_string())
            }
        }
        Scheme::Shallow => {
            // Shallow: {storage}/{monitor_id}/{event_id}/
            base.join(event_id.to_string())
        }
    }
}

/// Remove an event's on-disk media (its leaf directory, plus the `.{id}`
/// lookup symlink that the Deep scheme leaves beside the monitor directory).
///
/// Best-effort: a filesystem failure is logged but does not fail the caller —
/// the database row is the authoritative record of an event's existence, so a
/// rare orphaned directory is preferable to a delete that reports failure
/// after the row is already gone. An already-absent directory is not an error.
pub(crate) async fn delete_event_media(state: &AppState, event: &EventModel) -> AppResult<()> {
    let storage_path = resolve_event_storage_path(state, event).await?;
    remove_event_dir(&storage_path, event).await;
    Ok(())
}

/// Remove an event's on-disk media given an already-resolved `storage_path`
/// (its `Storages.Path` root). The retention reaper calls this directly with
/// the storage row it already loaded, avoiding a redundant lookup; the
/// interactive delete path reaches it via [`delete_event_media`]. Sharing this
/// one function is what keeps the reaper and the endpoint from ever computing
/// the event directory two different ways.
///
/// Best-effort: filesystem errors are logged, not returned. An already-absent
/// directory is not an error.
pub(crate) async fn remove_event_dir(storage_path: &str, event: &EventModel) {
    let dir = build_event_directory_path(
        storage_path,
        event.monitor_id,
        event.id,
        event.start_date_time,
        &event.scheme,
    );

    // Defence in depth: never remove the storage root or a bare monitor
    // directory. The path is built from typed integers/dates and a
    // traversal-checked root, so it always descends at least to the event's own
    // leaf — but refuse to proceed if that invariant is ever violated.
    let monitor_root = PathBuf::from(storage_path).join(event.monitor_id.to_string());
    if !dir.starts_with(&monitor_root) || dir == monitor_root {
        warn!(
            "Refusing to remove suspicious event directory {:?} (monitor root {:?})",
            dir, monitor_root
        );
        return;
    }

    match tokio::fs::remove_dir_all(&dir).await {
        Ok(()) => info!("Removed event {} media directory {:?}", event.id, dir),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            debug!(
                "Event {} media directory {:?} already absent",
                event.id, dir
            );
        }
        Err(e) => warn!(
            "Failed to remove event {} media directory {:?}: {e}",
            event.id, dir
        ),
    }

    // The Deep scheme also creates a `{storage}/{monitor}/.{id}` symlink that
    // points at the timestamped directory; remove it so it isn't left dangling.
    if matches!(event.scheme, Scheme::Deep) {
        let link = monitor_root.join(format!(".{}", event.id));
        if let Err(e) = tokio::fs::remove_file(&link).await {
            if e.kind() != ErrorKind::NotFound {
                debug!("Could not remove Deep-scheme symlink {:?}: {e}", link);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_dt() -> chrono::NaiveDateTime {
        chrono::NaiveDate::from_ymd_opt(2024, 3, 9)
            .unwrap()
            .and_hms_opt(14, 5, 6)
            .unwrap()
    }

    #[test]
    fn shallow_uses_event_id_leaf() {
        let p = build_event_directory_path("/events", 7, 467, Some(sample_dt()), &Scheme::Shallow);
        assert_eq!(p, PathBuf::from("/events/7/467"));
    }

    #[test]
    fn medium_uses_date_then_event_id() {
        let p = build_event_directory_path("/events", 7, 467, Some(sample_dt()), &Scheme::Medium);
        assert_eq!(p, PathBuf::from("/events/7/2024-03-09/467"));
    }

    #[test]
    fn deep_uses_two_digit_timestamp_path() {
        let p = build_event_directory_path("/events", 7, 467, Some(sample_dt()), &Scheme::Deep);
        assert_eq!(p, PathBuf::from("/events/7/24/03/09/14/05/06"));
    }

    #[test]
    fn deep_zero_pads_and_wraps_year_to_two_digits() {
        // Year 2008 -> "08"; single-digit month/day/time zero-padded.
        let dt = chrono::NaiveDate::from_ymd_opt(2008, 1, 2)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let p = build_event_directory_path("/srv", 1, 5, Some(dt), &Scheme::Deep);
        assert_eq!(p, PathBuf::from("/srv/1/08/01/02/00/00/00"));
    }

    #[test]
    fn deep_and_medium_fall_back_to_shallow_without_start_time() {
        assert_eq!(
            build_event_directory_path("/events", 7, 467, None, &Scheme::Deep),
            PathBuf::from("/events/7/467")
        );
        assert_eq!(
            build_event_directory_path("/events", 7, 467, None, &Scheme::Medium),
            PathBuf::from("/events/7/467")
        );
    }

    #[test]
    fn is_default_storage_treats_zero_and_null_as_default() {
        assert!(is_default_storage(None));
        assert!(is_default_storage(Some(0)));
        assert!(!is_default_storage(Some(1)));
        assert!(!is_default_storage(Some(7)));
    }

    fn mk_event(id: u64, monitor_id: u32, storage_id: Option<u16>, scheme: Scheme) -> EventModel {
        use crate::entity::sea_orm_active_enums::Orientation;
        use rust_decimal::Decimal;
        EventModel {
            id,
            monitor_id,
            storage_id,
            secondary_storage_id: None,
            name: "ev".into(),
            cause: None,
            start_date_time: None,
            end_date_time: None,
            width: 0,
            height: 0,
            length: Decimal::new(0, 0),
            frames: Some(0),
            alarm_frames: Some(0),
            default_video: "ev-video.mp4".into(),
            save_jpe_gs: None,
            tot_score: 0,
            avg_score: None,
            max_score: None,
            max_score_frame_id: None,
            archived: 0,
            videoed: 0,
            uploaded: 0,
            emailed: 0,
            messaged: 0,
            executed: 0,
            notes: None,
            state_id: 1,
            orientation: Orientation::Rotate0,
            disk_space: None,
            scheme,
            locked: 0,
            latitude: None,
            longitude: None,
        }
    }

    fn mk_storage(id: u16, path: String) -> crate::entity::storage::Model {
        use crate::entity::sea_orm_active_enums::StorageType;
        crate::entity::storage::Model {
            id,
            path,
            name: "store".into(),
            r#type: StorageType::Local,
            url: None,
            disk_space: None,
            scheme: Scheme::Shallow,
            server_id: None,
            do_delete: 1,
            enabled: 1,
        }
    }

    /// Deleting an event removes exactly its own leaf directory and leaves a
    /// sibling event's directory (under the same monitor) untouched.
    #[tokio::test]
    async fn delete_event_media_removes_only_the_event_leaf() {
        use sea_orm::{DatabaseBackend, MockDatabase};

        let tmp = tempfile::tempdir().unwrap();
        let monitor_id = 3u32;
        let event_id = 42u64;
        let monitor_dir = tmp.path().join(monitor_id.to_string());
        let event_dir = monitor_dir.join(event_id.to_string());
        let sibling_dir = monitor_dir.join("99");
        std::fs::create_dir_all(&event_dir).unwrap();
        std::fs::create_dir_all(&sibling_dir).unwrap();
        std::fs::write(event_dir.join("42-video.mp4"), b"video").unwrap();
        std::fs::write(sibling_dir.join("99-video.mp4"), b"other").unwrap();

        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<crate::entity::storage::Model, _, _>(vec![vec![mk_storage(
                1,
                tmp.path().to_string_lossy().into_owned(),
            )]])
            .into_connection();
        let state = AppState::for_test_with_db(db);

        let event = mk_event(event_id, monitor_id, Some(1), Scheme::Shallow);
        delete_event_media(&state, &event).await.unwrap();

        assert!(
            !event_dir.exists(),
            "the event's own directory must be removed"
        );
        assert!(
            sibling_dir.exists(),
            "a sibling event's directory must not be touched"
        );
    }

    /// An already-absent media directory is not an error (idempotent cleanup).
    #[tokio::test]
    async fn delete_event_media_tolerates_missing_directory() {
        use sea_orm::{DatabaseBackend, MockDatabase};

        let tmp = tempfile::tempdir().unwrap();
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<crate::entity::storage::Model, _, _>(vec![vec![mk_storage(
                1,
                tmp.path().to_string_lossy().into_owned(),
            )]])
            .into_connection();
        let state = AppState::for_test_with_db(db);

        // Nothing created on disk; the resolved directory does not exist.
        let event = mk_event(7, 1, Some(1), Scheme::Shallow);
        delete_event_media(&state, &event)
            .await
            .expect("absent media directory must not be an error");
    }
}
