//! Ingest of zm-next analysis EVENTs into ZoneMinder's `Events`/`Frames` rows.
//!
//! zm-next never touches the database — zm-api is the only writer. The source
//! router decodes monitor EVENTs off the stream socket and forwards them here
//! as [`MonitorEventEnvelope`]s; this task maps them onto the same Events model
//! the legacy `zmc`/`zma` daemons produce, so zm-next activity shows up on the
//! existing REST surface unchanged.
//!
//! ## Correlation model & id-assignment handshake
//!
//! A recording segment is the unit of correlation, anchored by an event id that
//! zm-api owns end to end:
//!
//! 1. `recording_opening` (store_event began a segment) → zm-api allocates (or
//!    adopts) the `Events` row, computes the Medium-scheme clip location from
//!    `Storage.Path` + the start date + the row id, and replies with a
//!    `0x11 Command` `assign_recording` carrying `event_id`, `dir` and
//!    `video_name`. store_event writes the clip exactly there, so ZoneMinder's
//!    own path derivation resolves it with no schema change.
//! 2. `detection` / `description` decorate the open row (alarm `Frames`, score
//!    aggregates, notes). They also open a row on their own if they arrive
//!    first, which `recording_opening` then adopts.
//! 3. `recording_saved` finalizes the row — preferring the `event_id` echoed
//!    back in its payload — with duration and end time, then closes the session.
//!
//! At most one event is open per monitor at a time, matching ZoneMinder's
//! per-monitor event model.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, NaiveDateTime, Utc};
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use super::detail::{
    DescriptionDetail, DetectionDetail, RecordingOpeningDetail, RecordingSavedDetail,
};
use crate::configure::zmnext::IngestConfig;
use crate::entity::sea_orm_active_enums::{FrameType, Scheme};
use crate::entity::{events, frames, monitors, storage};
use crate::error::AppResult;
use crate::streaming::source::{protocol, MonitorEvent, MonitorEventEnvelope};

/// In-memory aggregate for the event currently open on a monitor.
#[derive(Debug, Clone, PartialEq)]
struct OpenEvent {
    event_id: u64,
    start: NaiveDateTime,
    frames: u32,
    alarm_frames: u32,
    tot_score: u64,
    max_score: u16,
}

impl OpenEvent {
    fn new(event_id: u64, start: NaiveDateTime) -> Self {
        Self {
            event_id,
            start,
            frames: 0,
            alarm_frames: 0,
            tot_score: 0,
            max_score: 0,
        }
    }

    /// Fold one detection's score into the running aggregate and return the
    /// new frame number (1-based).
    fn record_detection(&mut self, score: u16) -> u32 {
        self.frames += 1;
        self.alarm_frames += 1;
        self.tot_score += score as u64;
        self.max_score = self.max_score.max(score);
        self.frames
    }

    fn avg_score(&self) -> u16 {
        if self.frames == 0 {
            0
        } else {
            (self.tot_score / self.frames as u64) as u16
        }
    }
}

/// Cached monitor attributes needed to open and locate an Events row, so the
/// common path does not re-query the Monitors/Storage tables per event.
#[derive(Debug, Clone)]
struct MonitorDims {
    width: u16,
    height: u16,
    storage_id: Option<u16>,
    /// Filesystem root of the resolved storage (for building the clip path).
    storage_path: Option<String>,
}

/// Consumes [`MonitorEventEnvelope`]s and writes Events/Frames rows.
pub struct EventIngestor {
    db: Arc<DatabaseConnection>,
    config: IngestConfig,
    open: HashMap<u32, OpenEvent>,
    dims: HashMap<u32, MonitorDims>,
}

impl EventIngestor {
    pub fn new(db: Arc<DatabaseConnection>, config: IngestConfig) -> Self {
        Self {
            db,
            config,
            open: HashMap::new(),
            dims: HashMap::new(),
        }
    }

    /// Drain the channel until all senders drop. Per-event DB failures are
    /// logged and skipped; the task itself only ends when the router is gone.
    pub async fn run(mut self, mut rx: mpsc::Receiver<MonitorEventEnvelope>) {
        info!("zm-next event ingest task started");
        while let Some(env) = rx.recv().await {
            if let Err(e) = self.handle(&env).await {
                warn!(
                    "zm-next ingest: monitor {} event {:#06x} failed: {}",
                    env.monitor_id, env.event.code, e
                );
            }
        }
        info!("zm-next event ingest task stopped (router dropped)");
    }

    async fn handle(&mut self, env: &MonitorEventEnvelope) -> AppResult<()> {
        let monitor_id = env.monitor_id;
        match env.event.code {
            protocol::EVENT_DETECTION => self.handle_detection(monitor_id, &env.event).await,
            protocol::EVENT_DESCRIPTION => self.handle_description(monitor_id, &env.event).await,
            protocol::EVENT_RECORDING_OPENING => {
                self.handle_recording_opening(monitor_id, &env.event, &env.reply)
                    .await
            }
            protocol::EVENT_RECORDING_SAVED => {
                self.handle_recording_saved(monitor_id, &env.event).await
            }
            // Lifecycle codes: an explicit return to a non-recording state ends
            // any open session; the rest are advisory and just logged.
            protocol::EVENT_STATE_CHANGED
            | protocol::EVENT_CONNECTION_FAILED
            | protocol::EVENT_CAPTURE_FAILED => {
                debug!(
                    "zm-next ingest: monitor {} lifecycle {:#06x} ({:?})",
                    monitor_id, env.event.code, env.event.state_name
                );
                Ok(())
            }
            other => {
                debug!("zm-next ingest: monitor {monitor_id} ignoring event {other:#06x}");
                Ok(())
            }
        }
    }

    async fn handle_detection(&mut self, monitor_id: u32, ev: &MonitorEvent) -> AppResult<()> {
        let detail = ev
            .json_detail
            .as_deref()
            .and_then(|j| DetectionDetail::parse(j).ok())
            .unwrap_or_default();
        let when = event_time(ev);
        let cause = non_empty(detail.cause_summary());
        let score = detail.peak_score();

        let event_id = self.ensure_open_event(monitor_id, when, cause).await?;

        // Fold the detection into the running aggregate, then persist a frame
        // and the updated event totals.
        let (frame_no, agg) = {
            let open = self
                .open
                .get_mut(&monitor_id)
                .expect("session opened above");
            let frame_no = open.record_detection(score);
            (frame_no, open.clone())
        };

        let delta = decimal_seconds((when - agg.start).num_milliseconds() as f64 / 1000.0);
        frames::ActiveModel {
            event_id: Set(event_id),
            frame_id: Set(frame_no),
            r#type: Set(FrameType::Alarm),
            time_stamp: Set(when.and_utc()),
            delta: Set(delta),
            score: Set(score),
            ..Default::default()
        }
        .insert(&*self.db)
        .await?;

        events::ActiveModel {
            id: Set(event_id),
            frames: Set(Some(agg.frames)),
            alarm_frames: Set(Some(agg.alarm_frames)),
            tot_score: Set(agg.tot_score as u32),
            avg_score: Set(Some(agg.avg_score())),
            max_score: Set(Some(agg.max_score)),
            ..Default::default()
        }
        .update(&*self.db)
        .await?;

        Ok(())
    }

    async fn handle_description(&mut self, monitor_id: u32, ev: &MonitorEvent) -> AppResult<()> {
        let detail = ev
            .json_detail
            .as_deref()
            .and_then(|j| DescriptionDetail::parse(j).ok())
            .unwrap_or_default();
        let when = event_time(ev);
        // A description on its own still implies activity worth a row.
        let event_id = self.ensure_open_event(monitor_id, when, None).await?;

        events::ActiveModel {
            id: Set(event_id),
            notes: Set(non_empty(detail.text)),
            ..Default::default()
        }
        .update(&*self.db)
        .await?;
        Ok(())
    }

    /// store_event opened a recording segment: allocate (or adopt) the event
    /// row, set its Medium-scheme video name, and reply with the event id +
    /// target directory so store_event writes the clip where ZoneMinder will
    /// later resolve it. This is the id-assignment handshake.
    async fn handle_recording_opening(
        &mut self,
        monitor_id: u32,
        ev: &MonitorEvent,
        reply: &crate::streaming::source::router::ControlReply,
    ) -> AppResult<()> {
        let detail = ev
            .json_detail
            .as_deref()
            .and_then(|j| RecordingOpeningDetail::parse(j).ok())
            .unwrap_or_default();
        let when = event_time(ev);

        // Adopt an event already opened by an earlier detection, else create
        // one now. Either way the id is what store_event will name the clip.
        let event_id = self.ensure_open_event(monitor_id, when, None).await?;
        let start = self.open.get(&monitor_id).map(|o| o.start).unwrap_or(when);
        let dims = self.monitor_dims(monitor_id).await?;

        let Some((dir, video_name)) = Self::clip_path(&dims, monitor_id, event_id, start) else {
            warn!(
                "zm-next ingest: monitor {monitor_id} has no resolvable storage path; \
                 cannot assign a clip path for event {event_id}"
            );
            return Ok(());
        };

        // Record the Medium-scheme video name so playback derives the same path.
        events::ActiveModel {
            id: Set(event_id),
            scheme: Set(Scheme::Medium),
            default_video: Set(video_name.clone()),
            ..Default::default()
        }
        .update(&*self.db)
        .await?;

        let command = serde_json::json!({
            "cmd": "assign_recording",
            "clip_token": detail.clip_token,
            "event_id": event_id,
            "dir": dir,
            "video_name": video_name,
        })
        .to_string();
        if !reply.send_command_json(&command) {
            warn!(
                "zm-next ingest: monitor {monitor_id} could not deliver assign_recording \
                 for event {event_id} (connection gone)"
            );
        } else {
            debug!("zm-next ingest: assigned event {event_id} → {dir}/{video_name}");
        }
        Ok(())
    }

    async fn handle_recording_saved(
        &mut self,
        monitor_id: u32,
        ev: &MonitorEvent,
    ) -> AppResult<()> {
        let detail = ev
            .json_detail
            .as_deref()
            .and_then(|j| RecordingSavedDetail::parse(j).ok())
            .unwrap_or_default();
        let end = event_time(ev);

        // Prefer the echoed event id from the handshake; fall back to the open
        // session, or index a standalone clip if neither is available (e.g.
        // zm-api restarted mid-event).
        let event_id = match detail
            .event_id
            .or_else(|| self.open.get(&monitor_id).map(|o| o.event_id))
        {
            Some(id) => id,
            None => self.ensure_open_event(monitor_id, end, None).await?,
        };

        let mut model = events::ActiveModel {
            id: Set(event_id),
            end_date_time: Set(Some(end)),
            default_video: Set(detail.file_name().to_string()),
            ..Default::default()
        };
        if let Some(d) = detail.duration {
            model.length = Set(decimal_seconds(d));
        }
        if let Some(f) = detail.frames {
            model.frames = Set(Some(f));
        }
        model.update(&*self.db).await?;

        info!(
            "zm-next ingest: monitor {monitor_id} indexed clip {:?} ({}s) → event {event_id}",
            detail.path,
            detail.duration.unwrap_or(0.0)
        );
        self.open.remove(&monitor_id);
        Ok(())
    }

    /// Return the open event's id for `monitor_id`, creating the Events row (and
    /// caching the session) if none is open.
    async fn ensure_open_event(
        &mut self,
        monitor_id: u32,
        start: NaiveDateTime,
        cause: Option<String>,
    ) -> AppResult<u64> {
        if let Some(open) = self.open.get(&monitor_id) {
            return Ok(open.event_id);
        }

        let dims = self.monitor_dims(monitor_id).await?;
        let model = events::ActiveModel {
            monitor_id: Set(monitor_id),
            name: Set(self.config.event_name.clone()),
            cause: Set(cause),
            start_date_time: Set(Some(start)),
            width: Set(dims.width),
            height: Set(dims.height),
            storage_id: Set(dims.storage_id),
            scheme: Set(Scheme::Shallow),
            length: Set(Decimal::ZERO),
            default_video: Set(String::new()),
            ..Default::default()
        }
        .insert(&*self.db)
        .await?;

        debug!(
            "zm-next ingest: opened event {} for monitor {monitor_id}",
            model.id
        );
        self.open
            .insert(monitor_id, OpenEvent::new(model.id, start));
        Ok(model.id)
    }

    async fn monitor_dims(&mut self, monitor_id: u32) -> AppResult<MonitorDims> {
        if let Some(d) = self.dims.get(&monitor_id) {
            return Ok(d.clone());
        }
        let dims = match monitors::Entity::find_by_id(monitor_id)
            .one(&*self.db)
            .await?
        {
            Some(m) => {
                let storage_id = self.config.default_storage_id.or(m.storage_id);
                let storage_path = self.storage_path(storage_id).await;
                MonitorDims {
                    width: m.width,
                    height: m.height,
                    storage_id,
                    storage_path,
                }
            }
            None => MonitorDims {
                width: 0,
                height: 0,
                storage_id: None,
                storage_path: None,
            },
        };
        self.dims.insert(monitor_id, dims.clone());
        Ok(dims)
    }

    /// Resolve a storage row's filesystem path: the given id, else the default
    /// (lowest-id) storage. `None` when no storage row is found.
    async fn storage_path(&self, storage_id: Option<u16>) -> Option<String> {
        use sea_orm::QueryOrder;

        let row = match storage_id {
            Some(sid) if sid != 0 => storage::Entity::find_by_id(sid).one(&*self.db).await.ok()?,
            _ => storage::Entity::find()
                .order_by_asc(storage::Column::Id)
                .one(&*self.db)
                .await
                .ok()?,
        }?;
        Some(row.path).filter(|p| !p.is_empty())
    }

    /// The directory + file name a clip for `event_id` must live at under the
    /// Medium scheme: `{storage}/{monitor}/{YYYY-MM-DD}/{event_id}` +
    /// `{event_id}-video.mp4`. This is exactly what ZoneMinder's playback
    /// derives, so handing it to `store_event` keeps the clip natively
    /// resolvable. `None` when the storage path is unknown.
    fn clip_path(
        dims: &MonitorDims,
        monitor_id: u32,
        event_id: u64,
        start: NaiveDateTime,
    ) -> Option<(String, String)> {
        let root = dims.storage_path.as_deref()?;
        let date = start.format("%Y-%m-%d");
        let dir = format!(
            "{}/{monitor_id}/{date}/{event_id}",
            root.trim_end_matches('/')
        );
        let video_name = format!("{event_id}-video.mp4");
        Some((dir, video_name))
    }
}

/// EVENT timestamp to use for a row: the wall-clock TLV when present, else now.
fn event_time(ev: &MonitorEvent) -> NaiveDateTime {
    ev.wall_clock_us
        .and_then(micros_to_naive)
        .unwrap_or_else(|| Utc::now().naive_utc())
}

/// Convert unix-epoch microseconds to a naive-UTC datetime.
fn micros_to_naive(us: u64) -> Option<NaiveDateTime> {
    let secs = (us / 1_000_000) as i64;
    let nanos = ((us % 1_000_000) * 1_000) as u32;
    DateTime::<Utc>::from_timestamp(secs, nanos).map(|dt| dt.naive_utc())
}

/// Seconds (possibly fractional) as a ZoneMinder `Decimal`, clamped at 0.
fn decimal_seconds(seconds: f64) -> Decimal {
    Decimal::from_f64_retain(seconds.max(0.0)).unwrap_or(Decimal::ZERO)
}

fn non_empty(s: String) -> Option<String> {
    if s.trim().is_empty() {
        None
    } else {
        Some(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_event_aggregates_scores() {
        let start = micros_to_naive(1_700_000_000_000_000).unwrap();
        let mut open = OpenEvent::new(42, start);

        assert_eq!(open.record_detection(80), 1);
        assert_eq!(open.record_detection(40), 2);
        assert_eq!(open.record_detection(90), 3);

        assert_eq!(open.frames, 3);
        assert_eq!(open.alarm_frames, 3);
        assert_eq!(open.tot_score, 210);
        assert_eq!(open.max_score, 90);
        assert_eq!(open.avg_score(), 70);
    }

    #[test]
    fn micros_round_trip_to_known_instant() {
        // 2023-11-14T22:13:20 UTC
        let dt = micros_to_naive(1_700_000_000_000_000).unwrap();
        assert_eq!(dt.and_utc().timestamp(), 1_700_000_000);
    }

    #[test]
    fn decimal_seconds_clamps_negative() {
        assert_eq!(decimal_seconds(-3.0), Decimal::ZERO);
        assert_eq!(decimal_seconds(12.5).to_string(), "12.5");
    }

    #[test]
    fn non_empty_filters_blank() {
        assert_eq!(non_empty("  ".to_string()), None);
        assert_eq!(non_empty("x".to_string()), Some("x".to_string()));
    }

    #[test]
    fn clip_path_builds_medium_scheme_layout() {
        let dims = MonitorDims {
            width: 1920,
            height: 1080,
            storage_id: Some(1),
            storage_path: Some("/var/lib/zm/events/".to_string()),
        };
        let start = micros_to_naive(1_700_000_000_000_000).unwrap(); // 2023-11-14 22:13:20 UTC
        let (dir, name) = EventIngestor::clip_path(&dims, 3, 512, start).unwrap();
        assert_eq!(dir, "/var/lib/zm/events/3/2023-11-14/512");
        assert_eq!(name, "512-video.mp4");

        // No storage path → cannot place the clip.
        let no_storage = MonitorDims {
            storage_path: None,
            ..dims
        };
        assert!(EventIngestor::clip_path(&no_storage, 3, 512, start).is_none());
    }
}
