use sea_orm::*;
use crate::entity::states::{Entity as States, Model as StateModel, ActiveModel};
use crate::error::AppResult;
use crate::dto::request::states::{CreateStateRequest, UpdateStateRequest};

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<StateModel>> {
    Ok(States::find().all(db).await?)
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<StateModel>> {
    Ok(States::find_by_id(id).one(db).await?)
}

pub async fn create(db: &DatabaseConnection, req: &CreateStateRequest) -> AppResult<StateModel> {
    let am = ActiveModel {
        id: Default::default(),
        name: Set(req.name.clone()),
        definition: Set(req.definition.clone()),
        is_active: Set(req.is_active),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(db: &DatabaseConnection, id: u32, req: &UpdateStateRequest) -> AppResult<Option<StateModel>> {
    let Some(model) = find_by_id(db, id).await? else { return Ok(None) };
    let mut am: ActiveModel = model.into();
    
    if let Some(v) = &req.name { am.name = Set(v.clone()); }
    if let Some(v) = &req.definition { am.definition = Set(v.clone()); }
    if let Some(v) = req.is_active { am.is_active = Set(v); }
    
    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    let res = States::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
