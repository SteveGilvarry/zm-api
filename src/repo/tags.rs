use sea_orm::*;
use crate::entity::tags::{Entity as Tags, Model as TagModel, ActiveModel, Column};
use crate::error::AppResult;
use crate::dto::request::tags::{CreateTagRequest, UpdateTagRequest};

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<TagModel>> {
    Ok(Tags::find().all(db).await?)
}

pub async fn find_by_id(db: &DatabaseConnection, id: u64) -> AppResult<Option<TagModel>> {
    Ok(Tags::find_by_id(id).one(db).await?)
}

pub async fn find_by_name(db: &DatabaseConnection, name: &str) -> AppResult<Option<TagModel>> {
    Ok(Tags::find()
        .filter(Column::Name.eq(name))
        .one(db)
        .await?)
}

pub async fn create(db: &DatabaseConnection, req: &CreateTagRequest) -> AppResult<TagModel> {
    let am = ActiveModel {
        id: Default::default(),
        name: Set(req.name.clone()),
        create_date: Set(req.create_date),
        ..Default::default()
    };
    Ok(am.insert(db).await?)
}

pub async fn update(db: &DatabaseConnection, id: u64, req: &UpdateTagRequest) -> AppResult<Option<TagModel>> {
    let Some(model) = find_by_id(db, id).await? else { return Ok(None) };
    let mut am: ActiveModel = model.into();
    
    if let Some(v) = &req.name { am.name = Set(v.clone()); }
    if let Some(v) = req.create_date { am.create_date = Set(Some(v)); }
    
    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u64) -> AppResult<bool> {
    let res = Tags::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
