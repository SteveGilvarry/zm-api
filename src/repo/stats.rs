use crate::dto::request::stats::{CreateStatRequest, UpdateStatRequest};
use crate::dto::PaginationParams;
use crate::entity::stats::{ActiveModel, Entity as Stats, Model as StatModel};
use crate::error::AppResult;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<StatModel>> {
    Ok(Stats::find().all(db).await?)
}

pub async fn find_paginated(
    db: &DatabaseConnection,
    params: &PaginationParams,
) -> AppResult<(Vec<StatModel>, u64)> {
    let paginator = Stats::find().paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<StatModel>> {
    Ok(Stats::find_by_id(id).one(db).await?)
}

pub async fn create(db: &DatabaseConnection, req: &CreateStatRequest) -> AppResult<StatModel> {
    let am = ActiveModel {
        id: Default::default(),
        monitor_id: Set(req.monitor_id),
        zone_id: Set(req.zone_id),
        event_id: Set(req.event_id),
        frame_id: Set(req.frame_id),
        pixel_diff: Set(req.pixel_diff),
        alarm_pixels: Set(req.alarm_pixels),
        filter_pixels: Set(req.filter_pixels),
        blob_pixels: Set(req.blob_pixels),
        blobs: Set(req.blobs),
        min_blob_size: Set(req.min_blob_size),
        max_blob_size: Set(req.max_blob_size),
        min_x: Set(req.min_x),
        max_x: Set(req.max_x),
        min_y: Set(req.min_y),
        max_y: Set(req.max_y),
        score: Set(req.score),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(
    db: &DatabaseConnection,
    id: u32,
    req: &UpdateStatRequest,
) -> AppResult<Option<StatModel>> {
    let Some(model) = find_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: ActiveModel = model.into();

    if let Some(v) = req.monitor_id {
        am.monitor_id = Set(v);
    }
    if let Some(v) = req.zone_id {
        am.zone_id = Set(v);
    }
    if let Some(v) = req.event_id {
        am.event_id = Set(v);
    }
    if let Some(v) = req.frame_id {
        am.frame_id = Set(v);
    }
    if let Some(v) = req.pixel_diff {
        am.pixel_diff = Set(v);
    }
    if let Some(v) = req.alarm_pixels {
        am.alarm_pixels = Set(v);
    }
    if let Some(v) = req.filter_pixels {
        am.filter_pixels = Set(v);
    }
    if let Some(v) = req.blob_pixels {
        am.blob_pixels = Set(v);
    }
    if let Some(v) = req.blobs {
        am.blobs = Set(v);
    }
    if let Some(v) = req.min_blob_size {
        am.min_blob_size = Set(v);
    }
    if let Some(v) = req.max_blob_size {
        am.max_blob_size = Set(v);
    }
    if let Some(v) = req.min_x {
        am.min_x = Set(v);
    }
    if let Some(v) = req.max_x {
        am.max_x = Set(v);
    }
    if let Some(v) = req.min_y {
        am.min_y = Set(v);
    }
    if let Some(v) = req.max_y {
        am.max_y = Set(v);
    }
    if let Some(v) = req.score {
        am.score = Set(v);
    }

    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    let res = Stats::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
