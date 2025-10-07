use sea_orm::*;
use crate::entity::object_types::{Entity as ObjectTypes, Model as ObjectTypeModel, ActiveModel};
use crate::error::AppResult;
use crate::dto::request::object_types::{CreateObjectTypeRequest, UpdateObjectTypeRequest};

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<ObjectTypeModel>> {
    Ok(ObjectTypes::find().all(db).await?)
}

pub async fn find_by_id(db: &DatabaseConnection, id: i32) -> AppResult<Option<ObjectTypeModel>> {
    Ok(ObjectTypes::find_by_id(id).one(db).await?)
}

pub async fn create(db: &DatabaseConnection, req: &CreateObjectTypeRequest) -> AppResult<ObjectTypeModel> {
    let am = ActiveModel {
        id: Default::default(),
        name: Set(req.name.clone()),
        human: Set(req.human.clone()),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(db: &DatabaseConnection, id: i32, req: &UpdateObjectTypeRequest) -> AppResult<Option<ObjectTypeModel>> {
    let Some(model) = find_by_id(db, id).await? else { return Ok(None) };
    let mut am: ActiveModel = model.into();
    
    if let Some(v) = &req.name { am.name = Set(Some(v.clone())); }
    if let Some(v) = &req.human { am.human = Set(Some(v.clone())); }
    
    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: i32) -> AppResult<bool> {
    let res = ObjectTypes::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
