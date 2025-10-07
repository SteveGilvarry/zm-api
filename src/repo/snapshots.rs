use sea_orm::*;
use crate::entity::snapshots::{Entity as Snapshots, Model as SnapshotModel, ActiveModel};
use crate::error::AppResult;
use crate::dto::request::snapshots::{CreateSnapshotRequest, UpdateSnapshotRequest};

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<SnapshotModel>> {
    Ok(Snapshots::find().all(db).await?)
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<SnapshotModel>> {
    Ok(Snapshots::find_by_id(id).one(db).await?)
}

pub async fn create(db: &DatabaseConnection, req: &CreateSnapshotRequest) -> AppResult<SnapshotModel> {
    let am = ActiveModel {
        id: Default::default(),
        name: Set(req.name.clone()),
        description: Set(req.description.clone()),
        created_by: Set(req.created_by),
        created_on: Set(req.created_on),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(db: &DatabaseConnection, id: u32, req: &UpdateSnapshotRequest) -> AppResult<Option<SnapshotModel>> {
    let Some(model) = find_by_id(db, id).await? else { return Ok(None) };
    let mut am: ActiveModel = model.into();
    
    if let Some(v) = &req.name { am.name = Set(Some(v.clone())); }
    if let Some(v) = &req.description { am.description = Set(Some(v.clone())); }
    if let Some(v) = req.created_by { am.created_by = Set(Some(v)); }
    if let Some(v) = req.created_on { am.created_on = Set(Some(v)); }
    
    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    let res = Snapshots::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
