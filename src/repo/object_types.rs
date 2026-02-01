use crate::dto::request::object_types::{CreateObjectTypeRequest, UpdateObjectTypeRequest};
use crate::dto::PaginationParams;
use crate::entity::object_types::{ActiveModel, Entity as ObjectTypes, Model as ObjectTypeModel};
use crate::error::AppResult;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<ObjectTypeModel>> {
    Ok(ObjectTypes::find().all(db).await?)
}

pub async fn find_paginated(
    db: &DatabaseConnection,
    params: &PaginationParams,
) -> AppResult<(Vec<ObjectTypeModel>, u64)> {
    let paginator = ObjectTypes::find().paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

pub async fn find_by_id(db: &DatabaseConnection, id: i32) -> AppResult<Option<ObjectTypeModel>> {
    Ok(ObjectTypes::find_by_id(id).one(db).await?)
}

pub async fn create(
    db: &DatabaseConnection,
    req: &CreateObjectTypeRequest,
) -> AppResult<ObjectTypeModel> {
    let am = ActiveModel {
        id: Default::default(),
        name: Set(req.name.clone()),
        human: Set(req.human.clone()),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(
    db: &DatabaseConnection,
    id: i32,
    req: &UpdateObjectTypeRequest,
) -> AppResult<Option<ObjectTypeModel>> {
    let Some(model) = find_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: ActiveModel = model.into();

    if let Some(v) = &req.name {
        am.name = Set(Some(v.clone()));
    }
    if let Some(v) = &req.human {
        am.human = Set(Some(v.clone()));
    }

    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: i32) -> AppResult<bool> {
    let res = ObjectTypes::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
