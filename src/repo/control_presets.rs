use sea_orm::*;
use crate::entity::control_presets::{Entity as ControlPresets, Model as ControlPresetModel, ActiveModel};
use crate::error::AppResult;
use crate::dto::request::control_presets::{CreateControlPresetRequest, UpdateControlPresetRequest};

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<ControlPresetModel>> {
    Ok(ControlPresets::find().all(db).await?)
}

pub async fn find_by_monitor_and_preset(db: &DatabaseConnection, monitor_id: u32, preset: u32) -> AppResult<Option<ControlPresetModel>> {
    Ok(ControlPresets::find_by_id((monitor_id, preset)).one(db).await?)
}

pub async fn find_by_monitor(db: &DatabaseConnection, monitor_id: u32) -> AppResult<Vec<ControlPresetModel>> {
    use crate::entity::control_presets::Column;
    Ok(ControlPresets::find()
        .filter(Column::MonitorId.eq(monitor_id))
        .all(db)
        .await?)
}

pub async fn create(db: &DatabaseConnection, req: &CreateControlPresetRequest) -> AppResult<ControlPresetModel> {
    let am = ActiveModel {
        monitor_id: Set(req.monitor_id),
        preset: Set(req.preset),
        label: Set(req.label.clone()),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(db: &DatabaseConnection, monitor_id: u32, preset: u32, req: &UpdateControlPresetRequest) -> AppResult<Option<ControlPresetModel>> {
    let Some(model) = find_by_monitor_and_preset(db, monitor_id, preset).await? else { return Ok(None) };
    let mut am: ActiveModel = model.into();
    
    if let Some(v) = &req.label { am.label = Set(v.clone()); }
    
    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, monitor_id: u32, preset: u32) -> AppResult<bool> {
    let res = ControlPresets::delete_by_id((monitor_id, preset)).exec(db).await?;
    Ok(res.rows_affected > 0)
}
