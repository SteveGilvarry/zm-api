use crate::dto::PaginationParams;
use crate::entity;
use crate::error::AppResult;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, ModelTrait,
    PaginatorTrait, QueryFilter,
};
use std::sync::Arc;

/// Monitor repository for database operations
pub struct MonitorRepository {
    db: Arc<DatabaseConnection>,
}

impl MonitorRepository {
    /// Create a new monitor repository
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Find all active monitors
    pub async fn find_all_active_monitors(&self) -> AppResult<Vec<entity::monitors::Model>> {
        let monitors = entity::monitors::Entity::find()
            .filter(entity::monitors::Column::Enabled.eq(1))
            .all(self.db.as_ref())
            .await?;
        Ok(monitors)
    }

    /// Find all monitors (active and inactive)
    pub async fn find_all(&self) -> AppResult<Vec<entity::monitors::Model>> {
        let monitors = entity::monitors::Entity::find()
            .all(self.db.as_ref())
            .await?;
        Ok(monitors)
    }

    /// Find monitor by ID
    pub async fn find_by_id(&self, id: i32) -> AppResult<Option<entity::monitors::Model>> {
        let monitor = entity::monitors::Entity::find_by_id(id as u32)
            .one(self.db.as_ref())
            .await?;
        Ok(monitor)
    }
}

/// Restrict a `monitors` query to an allowlist of ids.
///
/// `monitor_filter` is `None` for unrestricted callers and `Some(ids)` for a
/// row-level ACL allowlist (see [`crate::service::monitor_acl`]).
fn scoped(
    query: sea_orm::Select<entity::monitors::Entity>,
    monitor_filter: Option<&[u32]>,
) -> sea_orm::Select<entity::monitors::Entity> {
    match monitor_filter {
        None => query,
        Some(ids) => query.filter(entity::monitors::Column::Id.is_in(ids.iter().copied())),
    }
}

/// Find all monitors visible to the caller.
#[tracing::instrument(skip_all)]
pub async fn find_all<C>(
    conn: &C,
    monitor_filter: Option<&[u32]>,
) -> AppResult<Vec<entity::monitors::Model>>
where
    C: ConnectionTrait,
{
    let monitors = scoped(entity::monitors::Entity::find(), monitor_filter)
        .all(conn)
        .await?;
    Ok(monitors)
}

/// Find monitors with pagination, restricted to the caller's allowlist.
#[tracing::instrument(skip_all)]
pub async fn find_paginated<C>(
    conn: &C,
    params: &PaginationParams,
    monitor_filter: Option<&[u32]>,
) -> AppResult<(Vec<entity::monitors::Model>, u64)>
where
    C: ConnectionTrait,
{
    let paginator =
        scoped(entity::monitors::Entity::find(), monitor_filter).paginate(conn, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

/// Find a monitor by its ID
#[tracing::instrument(skip_all)]
pub async fn find_by_id<C>(conn: &C, id: u32) -> AppResult<Option<entity::monitors::Model>>
where
    C: ConnectionTrait,
{
    let monitor = entity::monitors::Entity::find_by_id(id).one(conn).await?;
    Ok(monitor)
}

/// Create a new monitor
#[tracing::instrument(skip_all)]
pub async fn create<C>(
    conn: &C,
    monitor: entity::monitors::ActiveModel,
) -> AppResult<entity::monitors::Model>
where
    C: ConnectionTrait,
{
    let monitor = monitor.insert(conn).await?;
    Ok(monitor)
}

/// Update an existing monitor
#[tracing::instrument(skip_all)]
pub async fn update<C>(
    conn: &C,
    monitor: entity::monitors::ActiveModel,
) -> AppResult<entity::monitors::Model>
where
    C: ConnectionTrait,
{
    let monitor = monitor.update(conn).await?;
    Ok(monitor)
}

/// Delete a monitor by ID
#[tracing::instrument(skip_all)]
pub async fn delete<C>(conn: &C, monitor: entity::monitors::Model) -> AppResult<()>
where
    C: ConnectionTrait,
{
    monitor.delete(conn).await?;
    Ok(())
}

/// Get streaming details for a monitor
#[tracing::instrument(skip_all)]
pub async fn get_streaming_details<C>(
    conn: &C,
    id: u32,
) -> AppResult<Option<entity::monitors::Model>>
where
    C: ConnectionTrait,
{
    let monitor = entity::monitors::Entity::find_by_id(id).one(conn).await?;
    Ok(monitor)
}

/// Read a monitor's `UseZmNext` flag.
///
/// Deliberately a standalone raw query rather than a column on the generated
/// `monitors` entity: SeaORM selects every modelled column, so adding
/// `UseZmNext` to the entity would break *every* monitor query against a DB
/// whose schema lacks it. The ZoneMinder fork owns that migration and has not
/// shipped it yet, so this query degrades to `false` (legacy capture) on a
/// missing column, missing row, or any other DB error — and starts returning
/// the real value automatically once the column exists.
pub async fn use_zmnext<C>(conn: &C, monitor_id: u32) -> bool
where
    C: ConnectionTrait,
{
    use sea_orm::{DbBackend, Statement};

    let stmt = Statement::from_sql_and_values(
        DbBackend::MySql,
        "SELECT `UseZmNext` AS use_zm_next FROM `Monitors` WHERE `Id` = ?",
        [monitor_id.into()],
    );
    match conn.query_one(stmt).await {
        Ok(Some(row)) => row
            .try_get::<i8>("", "use_zm_next")
            .map(|v| v != 0)
            .unwrap_or(false),
        _ => false,
    }
}

/// Set a monitor's `UseZmNext` flag. Raw UPDATE for the same reason
/// [`use_zmnext`] is a raw SELECT (the column is owned by the ZoneMinder fork
/// migration and is deliberately off the generated entity). Unlike the read,
/// this surfaces the DB error so a caller (e.g. "make it zm-next") can report
/// that the fork migration is required when the column is absent.
pub async fn set_use_zmnext<C>(
    conn: &C,
    monitor_id: u32,
    enabled: bool,
) -> Result<(), sea_orm::DbErr>
where
    C: ConnectionTrait,
{
    use sea_orm::{DbBackend, Statement};

    let stmt = Statement::from_sql_and_values(
        DbBackend::MySql,
        "UPDATE `Monitors` SET `UseZmNext` = ? WHERE `Id` = ?",
        [(enabled as i8).into(), monitor_id.into()],
    );
    conn.execute(stmt).await?;
    Ok(())
}
