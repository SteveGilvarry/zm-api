use sea_orm::*;
use crate::entity::monitor_presets::{Entity as MonitorPresets, Model as MonitorPresetModel, ActiveModel, Column};
use crate::error::AppResult;
use crate::dto::request::monitor_presets::{CreateMonitorPresetRequest, UpdateMonitorPresetRequest};

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<MonitorPresetModel>> {
    Ok(MonitorPresets::find().all(db).await?)
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<MonitorPresetModel>> {
    Ok(MonitorPresets::find_by_id(id).one(db).await?)
}

pub async fn find_by_model(db: &DatabaseConnection, model_id: u32) -> AppResult<Vec<MonitorPresetModel>> {
    Ok(MonitorPresets::find()
        .filter(Column::ModelId.eq(model_id))
        .all(db)
        .await?)
}

pub async fn create(db: &DatabaseConnection, req: &CreateMonitorPresetRequest) -> AppResult<MonitorPresetModel> {
    let am = ActiveModel {
        id: Default::default(),
        model_id: Set(req.model_id),
        name: Set(req.name.clone()),
        r#type: Set(req.r#type.clone()),
        device: Set(req.device.clone()),
        channel: Set(req.channel.clone()),
        format: Set(req.format),
        protocol: Set(req.protocol.clone()),
        method: Set(req.method.clone()),
        host: Set(req.host.clone()),
        port: Set(req.port.clone()),
        path: Set(req.path.clone()),
        sub_path: Set(req.sub_path.clone()),
        width: Set(req.width),
        height: Set(req.height),
        palette: Set(req.palette),
        max_fps: Set(req.max_fps),
        controllable: Set(req.controllable.unwrap_or(0)),
        control_id: Set(req.control_id.clone()),
        control_device: Set(req.control_device.clone()),
        control_address: Set(req.control_address.clone()),
        default_rate: Set(req.default_rate.unwrap_or(100)),
        default_scale: Set(req.default_scale.unwrap_or(100)),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(db: &DatabaseConnection, id: u32, req: &UpdateMonitorPresetRequest) -> AppResult<Option<MonitorPresetModel>> {
    let Some(model) = find_by_id(db, id).await? else { return Ok(None) };
    let mut am: ActiveModel = model.into();
    
    if let Some(v) = req.model_id { am.model_id = Set(Some(v)); }
    if let Some(v) = &req.name { am.name = Set(v.clone()); }
    if let Some(v) = &req.r#type { am.r#type = Set(v.clone()); }
    if let Some(v) = &req.device { am.device = Set(Some(v.clone())); }
    if let Some(v) = req.channel { am.channel = Set(Some(v)); }
    if let Some(v) = req.format { am.format = Set(Some(v)); }
    if let Some(v) = &req.protocol { am.protocol = Set(Some(v.clone())); }
    if let Some(v) = &req.method { am.method = Set(Some(v.clone())); }
    if let Some(v) = &req.host { am.host = Set(Some(v.clone())); }
    if let Some(v) = &req.port { am.port = Set(Some(v.clone())); }
    if let Some(v) = &req.path { am.path = Set(Some(v.clone())); }
    if let Some(v) = &req.sub_path { am.sub_path = Set(Some(v.clone())); }
    if let Some(v) = req.width { am.width = Set(Some(v)); }
    if let Some(v) = req.height { am.height = Set(Some(v)); }
    if let Some(v) = req.palette { am.palette = Set(Some(v)); }
    if let Some(v) = req.max_fps { am.max_fps = Set(Some(v)); }
    if let Some(v) = req.controllable { am.controllable = Set(v); }
    if let Some(v) = &req.control_id { am.control_id = Set(Some(v.clone())); }
    if let Some(v) = &req.control_device { am.control_device = Set(Some(v.clone())); }
    if let Some(v) = &req.control_address { am.control_address = Set(Some(v.clone())); }
    if let Some(v) = req.default_rate { am.default_rate = Set(v); }
    if let Some(v) = req.default_scale { am.default_scale = Set(v); }
    
    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    let res = MonitorPresets::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
