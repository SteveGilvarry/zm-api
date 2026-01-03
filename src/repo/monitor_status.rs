use sea_orm::*;
use crate::entity::monitor_status::{Entity as MonitorStatuses, Model as MonitorStatusModel, ActiveModel};
use crate::entity::sea_orm_active_enums::Status;
use crate::error::{AppResult, AppError};
use crate::dto::request::monitor_status::UpdateMonitorStatusRequest;
use rust_decimal::Decimal;
use std::str::FromStr;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<MonitorStatusModel>> {
    Ok(MonitorStatuses::find().all(db).await?)
}

pub async fn find_by_monitor_id(db: &DatabaseConnection, monitor_id: u32) -> AppResult<Option<MonitorStatusModel>> {
    Ok(MonitorStatuses::find_by_id(monitor_id).one(db).await?)
}

pub async fn update(db: &DatabaseConnection, monitor_id: u32, req: &UpdateMonitorStatusRequest) -> AppResult<Option<MonitorStatusModel>> {
    let Some(model) = find_by_monitor_id(db, monitor_id).await? else { return Ok(None) };
    let mut am: ActiveModel = model.into();
    
    if let Some(status_str) = &req.status {
        let status = match status_str.as_str() {
            "Unknown" => Status::Unknown,
            "NotRunning" => Status::NotRunning,
            "Running" => Status::Running,
            _ => return Err(AppError::BadRequestError(format!("Invalid status: {}", status_str))),
        };
        am.status = Set(status);
    }
    if let Some(v) = &req.capture_fps {
        let decimal = Decimal::from_str(v)
            .map_err(|e| AppError::BadRequestError(format!("Invalid capture_fps: {}", e)))?;
        am.capture_fps = Set(decimal);
    }
    if let Some(v) = &req.analysis_fps {
        let decimal = Decimal::from_str(v)
            .map_err(|e| AppError::BadRequestError(format!("Invalid analysis_fps: {}", e)))?;
        am.analysis_fps = Set(decimal);
    }
    if let Some(v) = req.capture_bandwidth { am.capture_bandwidth = Set(v); }
    
    let updated = am.update(db).await?;
    Ok(Some(updated))
}
