use sea_orm::*;
use crate::entity::devices::{Entity as Devices, Model as DeviceModel, ActiveModel};
use crate::error::AppResult;
use crate::dto::request::devices::{CreateDeviceRequest, UpdateDeviceRequest};

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<DeviceModel>> {
    Ok(Devices::find().all(db).await?)
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<DeviceModel>> {
    Ok(Devices::find_by_id(id).one(db).await?)
}

pub async fn create(db: &DatabaseConnection, req: &CreateDeviceRequest) -> AppResult<DeviceModel> {
    let am = ActiveModel {
        id: Default::default(),
        name: Set(req.name.clone()),
        r#type: Set(req.r#type.clone()),
        key_string: Set(req.key_string.clone()),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(db: &DatabaseConnection, id: u32, req: &UpdateDeviceRequest) -> AppResult<Option<DeviceModel>> {
    let Some(model) = find_by_id(db, id).await? else { return Ok(None) };
    let mut am: ActiveModel = model.into();
    
    if let Some(v) = &req.name { am.name = Set(v.clone()); }
    if let Some(v) = &req.r#type { am.r#type = Set(v.clone()); }
    if let Some(v) = &req.key_string { am.key_string = Set(v.clone()); }
    
    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    let res = Devices::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
