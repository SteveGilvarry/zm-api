use crate::dto::request::event_data::{CreateEventDataRequest, UpdateEventDataRequest};
use crate::entity::event_data::{
    ActiveModel, Column, Entity as EventData, Model as EventDataModel,
};
use crate::error::{AppError, AppResult};
use chrono::{DateTime, Utc};
use sea_orm::*;

pub async fn find_all(
    db: &DatabaseConnection,
    event_id: Option<u64>,
) -> AppResult<Vec<EventDataModel>> {
    let mut query = EventData::find();

    if let Some(eid) = event_id {
        query = query.filter(Column::EventId.eq(eid));
    }

    Ok(query.all(db).await?)
}

pub async fn find_by_id(db: &DatabaseConnection, id: u64) -> AppResult<Option<EventDataModel>> {
    Ok(EventData::find_by_id(id).one(db).await?)
}

pub async fn create(
    db: &DatabaseConnection,
    req: &CreateEventDataRequest,
) -> AppResult<EventDataModel> {
    let timestamp = req
        .timestamp
        .as_ref()
        .map(|s| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|e| AppError::BadRequestError(format!("Invalid timestamp format: {}", e)))
        })
        .transpose()?;

    let am = ActiveModel {
        id: Default::default(),
        event_id: Set(req.event_id),
        monitor_id: Set(req.monitor_id),
        frame_id: Set(req.frame_id),
        timestamp: Set(timestamp),
        data: Set(req.data.clone()),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(
    db: &DatabaseConnection,
    id: u64,
    req: &UpdateEventDataRequest,
) -> AppResult<Option<EventDataModel>> {
    let Some(model) = find_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: ActiveModel = model.into();

    if let Some(v) = &req.data {
        am.data = Set(Some(v.clone()));
    }

    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u64) -> AppResult<bool> {
    let res = EventData::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
