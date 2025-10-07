use crate::dto::request::frames::{CreateFrameRequest, UpdateFrameRequest};
use crate::entity::frames::{ActiveModel, Column, Entity as FrameEntity, Model as FrameModel};
use crate::error::AppResult;
use chrono::DateTime;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

/// Find all frames, optionally filtered by event_id
pub async fn find_all(
    db: &DatabaseConnection,
    event_id: Option<u64>,
) -> AppResult<Vec<FrameModel>> {
    let mut query = FrameEntity::find();

    if let Some(eid) = event_id {
        query = query.filter(Column::EventId.eq(eid));
    }

    let frames = query.order_by_asc(Column::FrameId).all(db).await?;
    Ok(frames)
}

/// Find frame by id
pub async fn find_by_id(db: &DatabaseConnection, id: u64) -> AppResult<Option<FrameModel>> {
    let frame = FrameEntity::find_by_id(id).one(db).await?;
    Ok(frame)
}

/// Create a new frame
pub async fn create(db: &DatabaseConnection, req: &CreateFrameRequest) -> AppResult<FrameModel> {
    // Parse timestamp
    let time_stamp = DateTime::parse_from_rfc3339(&req.time_stamp)
        .map_err(|e| crate::error::AppError::BadRequestError(format!("Invalid timestamp: {}", e)))?
        .naive_utc();

    let frame = ActiveModel {
        event_id: Set(req.event_id),
        frame_id: Set(req.frame_id),
        r#type: Set(req.r#type.clone()),
        time_stamp: Set(time_stamp.and_utc()),
        delta: Set(req.delta),
        score: Set(req.score),
        ..Default::default()
    };

    let result = frame.insert(db).await?;
    Ok(result)
}

/// Update frame by id
pub async fn update(
    db: &DatabaseConnection,
    id: u64,
    req: &UpdateFrameRequest,
) -> AppResult<FrameModel> {
    let frame = FrameEntity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFoundError(crate::error::Resource {
                resource_type: crate::error::ResourceType::File,
                details: vec![("id".to_string(), id.to_string())],
            })
        })?;

    let mut frame: ActiveModel = frame.into();

    if let Some(ref t) = req.r#type {
        frame.r#type = Set(t.clone());
    }
    if let Some(s) = req.score {
        frame.score = Set(s);
    }

    let updated = frame.update(db).await?;
    Ok(updated)
}

/// Delete frame by id
pub async fn delete_by_id(db: &DatabaseConnection, id: u64) -> AppResult<()> {
    FrameEntity::delete_by_id(id).exec(db).await?;
    Ok(())
}
