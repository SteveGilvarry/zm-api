use crate::dto::request::user_preferences::{
    CreateUserPreferenceRequest, UpdateUserPreferenceRequest,
};
use crate::dto::PaginationParams;
use crate::entity::user_preferences::{
    ActiveModel, Column, Entity as UserPreferences, Model as UserPreferenceModel,
};
use crate::error::AppResult;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<UserPreferenceModel>> {
    Ok(UserPreferences::find().all(db).await?)
}

pub async fn find_paginated(
    db: &DatabaseConnection,
    params: &PaginationParams,
) -> AppResult<(Vec<UserPreferenceModel>, u64)> {
    let paginator = UserPreferences::find().paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

pub async fn find_by_id(
    db: &DatabaseConnection,
    id: u32,
) -> AppResult<Option<UserPreferenceModel>> {
    Ok(UserPreferences::find_by_id(id).one(db).await?)
}

pub async fn find_by_user(
    db: &DatabaseConnection,
    user_id: u32,
) -> AppResult<Vec<UserPreferenceModel>> {
    Ok(UserPreferences::find()
        .filter(Column::UserId.eq(user_id))
        .all(db)
        .await?)
}

pub async fn create(
    db: &DatabaseConnection,
    req: &CreateUserPreferenceRequest,
) -> AppResult<UserPreferenceModel> {
    let am = ActiveModel {
        id: Default::default(),
        user_id: Set(req.user_id),
        name: Set(req.name.clone()),
        value: Set(req.value.clone()),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(
    db: &DatabaseConnection,
    id: u32,
    req: &UpdateUserPreferenceRequest,
) -> AppResult<Option<UserPreferenceModel>> {
    let Some(model) = find_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: ActiveModel = model.into();

    if let Some(v) = req.user_id {
        am.user_id = Set(v);
    }
    if let Some(v) = &req.name {
        am.name = Set(Some(v.clone()));
    }
    if let Some(v) = &req.value {
        am.value = Set(Some(v.clone()));
    }

    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    let res = UserPreferences::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
