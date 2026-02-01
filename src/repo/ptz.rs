//! PTZ repository functions for database operations

use crate::entity::controls::{Entity as Controls, Model as ControlModel};
use crate::entity::monitors::{Entity as Monitors, Model as MonitorModel};
use crate::error::AppResult;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

/// Get a monitor with its PTZ control configuration
#[tracing::instrument(skip(db))]
pub async fn get_monitor_with_control(
    db: &DatabaseConnection,
    monitor_id: u32,
) -> AppResult<Option<(MonitorModel, Option<ControlModel>)>> {
    // Get the monitor
    let monitor = Monitors::find_by_id(monitor_id).one(db).await?;

    let Some(monitor) = monitor else {
        return Ok(None);
    };

    // If monitor has a control_id, fetch the control
    let control = if let Some(control_id) = monitor.control_id {
        Controls::find_by_id(control_id).one(db).await?
    } else {
        None
    };

    Ok(Some((monitor, control)))
}

/// Get all controllable monitors with their control configurations
#[tracing::instrument(skip(db))]
pub async fn get_all_controllable_monitors(
    db: &DatabaseConnection,
) -> AppResult<Vec<(MonitorModel, ControlModel)>> {
    use crate::entity::monitors::Column as MonitorColumn;

    // Get all monitors that are controllable and have a control_id
    let monitors = Monitors::find()
        .filter(MonitorColumn::Controllable.eq(1u8))
        .filter(MonitorColumn::ControlId.is_not_null())
        .all(db)
        .await?;

    // Fetch controls for each monitor
    let mut results = Vec::new();
    for monitor in monitors {
        if let Some(control_id) = monitor.control_id {
            if let Some(control) = Controls::find_by_id(control_id).one(db).await? {
                results.push((monitor, control));
            }
        }
    }

    Ok(results)
}

/// Get control by ID
#[tracing::instrument(skip(db))]
pub async fn get_control_by_id(
    db: &DatabaseConnection,
    control_id: u32,
) -> AppResult<Option<ControlModel>> {
    Ok(Controls::find_by_id(control_id).one(db).await?)
}

/// Check if a monitor is controllable
#[tracing::instrument(skip(db))]
pub async fn is_monitor_controllable(db: &DatabaseConnection, monitor_id: u32) -> AppResult<bool> {
    let monitor = Monitors::find_by_id(monitor_id).one(db).await?;

    Ok(monitor
        .map(|m| m.controllable != 0 && m.control_id.is_some())
        .unwrap_or(false))
}
