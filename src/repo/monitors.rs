use crate::entity;
use crate::error::AppResult;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, ModelTrait,
    PaginatorTrait, QueryFilter, QueryOrder,
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

/// Find all monitors
#[tracing::instrument(skip_all)]
pub async fn find_all<C>(conn: &C) -> AppResult<Vec<entity::monitors::Model>>
where
    C: ConnectionTrait,
{
    let monitors = entity::monitors::Entity::find().all(conn).await?;
    Ok(monitors)
}

/// Find all monitors with pagination
#[tracing::instrument(skip_all)]
pub async fn find_all_paginated<C>(
    conn: &C,
    page: u64,
    page_size: u64,
) -> AppResult<(Vec<entity::monitors::Model>, u64)>
where
    C: ConnectionTrait,
{
    let paginator = entity::monitors::Entity::find()
        .order_by_asc(entity::monitors::Column::Id)
        .paginate(conn, page_size);

    let total = paginator.num_items().await?;
    let monitors = paginator.fetch_page(page.saturating_sub(1)).await?;

    Ok((monitors, total))
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
