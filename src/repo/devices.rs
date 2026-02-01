use crate::dto::request::devices::{CreateDeviceRequest, UpdateDeviceRequest};
use crate::dto::PaginationParams;
use crate::entity::devices::{ActiveModel, Entity as Devices, Model as DeviceModel};
use crate::error::AppResult;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<DeviceModel>> {
    Ok(Devices::find().all(db).await?)
}

pub async fn find_paginated(
    db: &DatabaseConnection,
    params: &PaginationParams,
) -> AppResult<(Vec<DeviceModel>, u64)> {
    let paginator = Devices::find().paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
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

pub async fn update(
    db: &DatabaseConnection,
    id: u32,
    req: &UpdateDeviceRequest,
) -> AppResult<Option<DeviceModel>> {
    let Some(model) = find_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: ActiveModel = model.into();

    if let Some(v) = &req.name {
        am.name = Set(v.clone());
    }
    if let Some(v) = &req.r#type {
        am.r#type = Set(v.clone());
    }
    if let Some(v) = &req.key_string {
        am.key_string = Set(v.clone());
    }

    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    let res = Devices::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
