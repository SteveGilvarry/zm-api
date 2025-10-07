use sea_orm::*;
use chrono::NaiveDateTime;
use crate::entity::reports::{Entity as Reports, Model as ReportModel, ActiveModel};
use crate::error::{AppResult, AppError};
use crate::dto::request::reports::{CreateReportRequest, UpdateReportRequest};

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<ReportModel>> {
    Ok(Reports::find().all(db).await?)
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<ReportModel>> {
    Ok(Reports::find_by_id(id).one(db).await?)
}

pub async fn create(db: &DatabaseConnection, req: &CreateReportRequest) -> AppResult<ReportModel> {
    let start_date_time = req.start_date_time.as_ref()
        .map(|s| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ")
            .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")))
        .transpose()
        .map_err(|e| AppError::BadRequestError(format!("Invalid start_date_time format: {}", e)))?;
    
    let end_date_time = req.end_date_time.as_ref()
        .map(|s| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ")
            .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")))
        .transpose()
        .map_err(|e| AppError::BadRequestError(format!("Invalid end_date_time format: {}", e)))?;
    
    let am = ActiveModel {
        id: Default::default(),
        name: Set(req.name.clone()),
        filter_id: Set(req.filter_id),
        start_date_time: Set(start_date_time),
        end_date_time: Set(end_date_time),
        interval: Set(req.interval),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(db: &DatabaseConnection, id: u32, req: &UpdateReportRequest) -> AppResult<Option<ReportModel>> {
    let Some(model) = find_by_id(db, id).await? else { return Ok(None) };
    let mut am: ActiveModel = model.into();
    
    if let Some(v) = &req.name { am.name = Set(Some(v.clone())); }
    if let Some(v) = req.filter_id { am.filter_id = Set(Some(v)); }
    if let Some(v) = req.interval { am.interval = Set(Some(v)); }
    
    if let Some(s) = &req.start_date_time {
        let dt = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ")
            .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
            .map_err(|e| AppError::BadRequestError(format!("Invalid start_date_time format: {}", e)))?;
        am.start_date_time = Set(Some(dt));
    }
    
    if let Some(s) = &req.end_date_time {
        let dt = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ")
            .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
            .map_err(|e| AppError::BadRequestError(format!("Invalid end_date_time format: {}", e)))?;
        am.end_date_time = Set(Some(dt));
    }
    
    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    let res = Reports::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
