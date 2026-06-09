use crate::dto::request::snapshots_events::CreateSnapshotEventRequest;
use crate::dto::PaginationParams;
use crate::entity::snapshots_events::{
    ActiveModel, Column, Entity as SnapshotsEvents, Model as SnapshotEventModel,
};
use crate::error::AppResult;
use sea_orm::*;

/// Subquery selecting event ids that belong to an allowlist of monitors.
/// Used to row-level-ACL-filter `Snapshots_Events`, which link to monitors
/// only via their parent event.
fn events_for_monitors(ids: &[u32]) -> sea_orm::sea_query::SelectStatement {
    use crate::entity::events;
    use sea_orm::sea_query::Query;
    Query::select()
        .column(events::Column::Id)
        .from(events::Entity)
        .and_where(events::Column::MonitorId.is_in(ids.iter().copied()))
        .to_owned()
}

pub async fn find_all(
    db: &DatabaseConnection,
    monitor_filter: Option<&[u32]>,
) -> AppResult<Vec<SnapshotEventModel>> {
    let mut query = SnapshotsEvents::find();
    if let Some(ids) = monitor_filter {
        query = query.filter(Column::EventId.in_subquery(events_for_monitors(ids)));
    }
    Ok(query.all(db).await?)
}

pub async fn find_paginated(
    db: &DatabaseConnection,
    params: &PaginationParams,
    monitor_filter: Option<&[u32]>,
) -> AppResult<(Vec<SnapshotEventModel>, u64)> {
    let mut query = SnapshotsEvents::find();
    if let Some(ids) = monitor_filter {
        query = query.filter(Column::EventId.in_subquery(events_for_monitors(ids)));
    }
    let paginator = query.paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<SnapshotEventModel>> {
    Ok(SnapshotsEvents::find_by_id(id).one(db).await?)
}

pub async fn find_by_snapshot_id(
    db: &DatabaseConnection,
    snapshot_id: u32,
    monitor_filter: Option<&[u32]>,
) -> AppResult<Vec<SnapshotEventModel>> {
    let mut query = SnapshotsEvents::find().filter(Column::SnapshotId.eq(snapshot_id));
    if let Some(ids) = monitor_filter {
        query = query.filter(Column::EventId.in_subquery(events_for_monitors(ids)));
    }
    Ok(query.all(db).await?)
}

pub async fn find_by_event_id(
    db: &DatabaseConnection,
    event_id: u64,
) -> AppResult<Vec<SnapshotEventModel>> {
    Ok(SnapshotsEvents::find()
        .filter(Column::EventId.eq(event_id))
        .all(db)
        .await?)
}

pub async fn create(
    db: &DatabaseConnection,
    req: &CreateSnapshotEventRequest,
) -> AppResult<SnapshotEventModel> {
    let am = ActiveModel {
        id: Default::default(),
        snapshot_id: Set(req.snapshot_id),
        event_id: Set(req.event_id),
    };
    Ok(am.insert(db).await?)
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    let res = SnapshotsEvents::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
