//! DB query layer for the zm_api-owned `event_synopsis` table.

use sea_orm::*;

use crate::entity::event_synopsis;
use crate::entity::prelude::EventSynopsis;
use crate::entity::sea_orm_active_enums::SynopsisStatus;

/// Find a synopsis row by its own primary key.
pub async fn find_by_id(
    db: &DatabaseConnection,
    id: u64,
) -> Result<Option<event_synopsis::Model>, DbErr> {
    EventSynopsis::find_by_id(id).one(db).await
}

/// Find the synopsis row for a source event id (the reconciled key).
pub async fn find_by_event_id(
    db: &DatabaseConnection,
    event_id: u64,
) -> Result<Option<event_synopsis::Model>, DbErr> {
    EventSynopsis::find()
        .filter(event_synopsis::Column::EventId.eq(event_id))
        .one(db)
        .await
}

/// Find the synopsis row by the `(monitor_id, clip_token)` fallback key, used
/// before `event_id` is reconciled.
pub async fn find_by_monitor_clip(
    db: &DatabaseConnection,
    monitor_id: u32,
    clip_token: &str,
) -> Result<Option<event_synopsis::Model>, DbErr> {
    EventSynopsis::find()
        .filter(event_synopsis::Column::MonitorId.eq(monitor_id))
        .filter(event_synopsis::Column::ClipToken.eq(clip_token))
        .one(db)
        .await
}

/// Insert a new synopsis row, returning the persisted model.
pub async fn insert(
    db: &DatabaseConnection,
    model: event_synopsis::ActiveModel,
) -> Result<event_synopsis::Model, DbErr> {
    model.insert(db).await
}

/// Transition a row's render status, optionally setting the cached artifact
/// path (set on `Ready`, cleared otherwise is the caller's choice).
pub async fn update_status(
    db: &DatabaseConnection,
    id: u64,
    status: SynopsisStatus,
    rendered_path: Option<String>,
) -> Result<(), DbErr> {
    let model = event_synopsis::ActiveModel {
        id: Set(id),
        status: Set(status),
        rendered_path: Set(rendered_path),
        ..Default::default()
    };
    model.update(db).await?;
    Ok(())
}

/// Rows for a monitor created within `[from, to]`, newest first, capped at
/// `limit` (range/overview synopsis). The cap bounds cost on busy cameras —
/// callers log when more were available than the cap allowed.
pub async fn find_by_monitor_created_between(
    db: &DatabaseConnection,
    monitor_id: u32,
    from: chrono::NaiveDateTime,
    to: chrono::NaiveDateTime,
    limit: u64,
) -> Result<Vec<event_synopsis::Model>, DbErr> {
    EventSynopsis::find()
        .filter(event_synopsis::Column::MonitorId.eq(monitor_id))
        .filter(event_synopsis::Column::CreatedAt.gte(from))
        .filter(event_synopsis::Column::CreatedAt.lte(to))
        .order_by_desc(event_synopsis::Column::CreatedAt)
        .limit(limit)
        .all(db)
        .await
}

/// Count of rows for a monitor in `[from, to]` (to report when the overview cap
/// dropped some).
pub async fn count_by_monitor_created_between(
    db: &DatabaseConnection,
    monitor_id: u32,
    from: chrono::NaiveDateTime,
    to: chrono::NaiveDateTime,
) -> Result<u64, DbErr> {
    EventSynopsis::find()
        .filter(event_synopsis::Column::MonitorId.eq(monitor_id))
        .filter(event_synopsis::Column::CreatedAt.gte(from))
        .filter(event_synopsis::Column::CreatedAt.lte(to))
        .count(db)
        .await
}

/// All rows whose `expires_at` is in the past relative to `now` (retention job).
pub async fn find_expired(
    db: &DatabaseConnection,
    now: chrono::NaiveDateTime,
) -> Result<Vec<event_synopsis::Model>, DbErr> {
    EventSynopsis::find()
        .filter(event_synopsis::Column::ExpiresAt.is_not_null())
        .filter(event_synopsis::Column::ExpiresAt.lt(now))
        .all(db)
        .await
}

/// Delete a synopsis row by id (retention job, after the artifact is removed).
pub async fn delete_by_id(db: &DatabaseConnection, id: u64) -> Result<(), DbErr> {
    EventSynopsis::delete_by_id(id).exec(db).await?;
    Ok(())
}
