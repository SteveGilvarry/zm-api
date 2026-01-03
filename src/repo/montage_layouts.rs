use crate::dto::request::montage_layouts::{
    CreateMontageLayoutRequest, UpdateMontageLayoutRequest,
};
use crate::entity::montage_layouts::{
    ActiveModel, Column, Entity as MontageLayouts, Model as MontageLayoutModel,
};
use crate::error::AppResult;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<MontageLayoutModel>> {
    Ok(MontageLayouts::find().all(db).await?)
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<MontageLayoutModel>> {
    Ok(MontageLayouts::find_by_id(id).one(db).await?)
}

pub async fn find_by_user(
    db: &DatabaseConnection,
    user_id: u32,
) -> AppResult<Vec<MontageLayoutModel>> {
    Ok(MontageLayouts::find()
        .filter(Column::UserId.eq(user_id))
        .all(db)
        .await?)
}

pub async fn create(
    db: &DatabaseConnection,
    req: &CreateMontageLayoutRequest,
) -> AppResult<MontageLayoutModel> {
    let am = ActiveModel {
        id: Default::default(),
        name: Set(req.name.clone()),
        user_id: Set(req.user_id),
        positions: Set(req.positions.clone()),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(
    db: &DatabaseConnection,
    id: u32,
    req: &UpdateMontageLayoutRequest,
) -> AppResult<Option<MontageLayoutModel>> {
    let Some(model) = find_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: ActiveModel = model.into();

    if let Some(v) = &req.name {
        am.name = Set(v.clone());
    }
    if let Some(v) = req.user_id {
        am.user_id = Set(v);
    }
    if let Some(v) = &req.positions {
        am.positions = Set(Some(v.clone()));
    }

    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    let res = MontageLayouts::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
