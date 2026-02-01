use crate::dto::request::triggers_x10::{CreateTriggerX10Request, UpdateTriggerX10Request};
use crate::dto::PaginationParams;
use crate::entity::triggers_x10::{ActiveModel, Entity as TriggersX10, Model as TriggerX10Model};
use crate::error::AppResult;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<TriggerX10Model>> {
    Ok(TriggersX10::find().all(db).await?)
}

pub async fn find_paginated(
    db: &DatabaseConnection,
    params: &PaginationParams,
) -> AppResult<(Vec<TriggerX10Model>, u64)> {
    let paginator = TriggersX10::find().paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

pub async fn find_by_id(
    db: &DatabaseConnection,
    monitor_id: u32,
) -> AppResult<Option<TriggerX10Model>> {
    Ok(TriggersX10::find_by_id(monitor_id).one(db).await?)
}

pub async fn create(
    db: &DatabaseConnection,
    req: &CreateTriggerX10Request,
) -> AppResult<TriggerX10Model> {
    let am = ActiveModel {
        monitor_id: Set(req.monitor_id),
        activation: Set(req.activation.clone()),
        alarm_input: Set(req.alarm_input.clone()),
        alarm_output: Set(req.alarm_output.clone()),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(
    db: &DatabaseConnection,
    monitor_id: u32,
    req: &UpdateTriggerX10Request,
) -> AppResult<Option<TriggerX10Model>> {
    let Some(model) = find_by_id(db, monitor_id).await? else {
        return Ok(None);
    };
    let mut am: ActiveModel = model.into();

    if let Some(v) = &req.activation {
        am.activation = Set(Some(v.clone()));
    }
    if let Some(v) = &req.alarm_input {
        am.alarm_input = Set(Some(v.clone()));
    }
    if let Some(v) = &req.alarm_output {
        am.alarm_output = Set(Some(v.clone()));
    }

    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, monitor_id: u32) -> AppResult<bool> {
    let res = TriggersX10::delete_by_id(monitor_id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
