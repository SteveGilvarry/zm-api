use crate::dto::PaginationParams;
use crate::entity::zones::{Entity as Zones, Model as ZoneModel};
use crate::error::AppResult;
use sea_orm::*;

pub async fn find_by_monitor(
    db: &DatabaseConnection,
    monitor_id: u32,
) -> AppResult<Vec<ZoneModel>> {
    let zones = Zones::find()
        .filter(crate::entity::zones::Column::MonitorId.eq(monitor_id))
        .all(db)
        .await?;
    Ok(zones)
}

pub async fn find_by_monitor_paginated(
    db: &DatabaseConnection,
    monitor_id: u32,
    params: &PaginationParams,
) -> AppResult<(Vec<ZoneModel>, u64)> {
    let paginator = Zones::find()
        .filter(crate::entity::zones::Column::MonitorId.eq(monitor_id))
        .paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<ZoneModel>> {
    let zone = Zones::find_by_id(id).one(db).await?;
    Ok(zone)
}

/// Rename a zone and/or replace its polygon. Kept for callers that only move
/// the geometry; [`update`] takes every setting.
pub async fn update_coords(
    db: &DatabaseConnection,
    id: u32,
    name: Option<String>,
    coords: Option<String>,
) -> AppResult<Option<ZoneModel>> {
    let req = crate::dto::request::zones::UpdateZoneRequest {
        name,
        coords,
        ..Default::default()
    };
    update(db, id, &req).await
}

fn bad_request(msg: impl Into<String>) -> crate::error::AppError {
    crate::error::AppError::BadRequestError(msg.into())
}

fn parse_zone_type(s: &str) -> AppResult<crate::entity::sea_orm_active_enums::ZoneType> {
    use crate::entity::sea_orm_active_enums::ZoneType::*;
    Ok(match s.to_lowercase().as_str() {
        "active" => Active,
        "inclusive" => Inclusive,
        "exclusive" => Exclusive,
        "preclusive" => Preclusive,
        "inactive" => Inactive,
        "privacy" => Privacy,
        _ => {
            return Err(bad_request(format!(
            "type {s:?} is not one of Active, Inclusive, Exclusive, Preclusive, Inactive, Privacy"
        )))
        }
    })
}

fn parse_units(s: &str) -> AppResult<crate::entity::sea_orm_active_enums::Units> {
    use crate::entity::sea_orm_active_enums::Units::*;
    Ok(match s.to_lowercase().as_str() {
        "pixels" => Pixels,
        "percent" => Percent,
        _ => return Err(bad_request(format!("units {s:?} is not Pixels or Percent"))),
    })
}

fn parse_check_method(s: &str) -> AppResult<crate::entity::sea_orm_active_enums::CheckMethod> {
    use crate::entity::sea_orm_active_enums::CheckMethod::*;
    Ok(match s.to_lowercase().as_str() {
        "alarmedpixels" => AlarmedPixels,
        "filteredpixels" => FilteredPixels,
        "blobs" => Blobs,
        _ => {
            return Err(bad_request(format!(
                "check_method {s:?} is not AlarmedPixels, FilteredPixels or Blobs"
            )))
        }
    })
}

/// Area and point count for a polygon, or a 400 naming what's wrong.
fn polygon(coords: &str) -> AppResult<(u32, u8)> {
    let points = crate::service::polygon::parse_coords(coords).ok_or_else(|| {
        bad_request(format!(
            "coords {coords:?} is not a polygon: expected at least three space-separated x,y pairs"
        ))
    })?;
    let count = u8::try_from(points.len())
        .map_err(|_| bad_request(format!("coords has {} points; at most 255", points.len())))?;
    Ok((crate::service::polygon::polygon_area(&points), count))
}

fn to_decimal(v: f64) -> Option<rust_decimal::Decimal> {
    rust_decimal::Decimal::from_f64_retain(v).map(|d| d.round_dp(2))
}

/// Refuse a zone whose minimums exceed their maximums.
fn check_ranges(z: &ZoneModel) -> AppResult<()> {
    fn pair<T: PartialOrd + std::fmt::Display>(
        name: &str,
        min: Option<T>,
        max: Option<T>,
    ) -> AppResult<()> {
        match (min, max) {
            (Some(lo), Some(hi)) if lo > hi => Err(bad_request(format!(
                "min_{name} ({lo}) is greater than max_{name} ({hi})"
            ))),
            _ => Ok(()),
        }
    }
    pair(
        "pixel_threshold",
        z.min_pixel_threshold,
        z.max_pixel_threshold,
    )?;
    pair("alarm_pixels", z.min_alarm_pixels, z.max_alarm_pixels)?;
    pair("filter_pixels", z.min_filter_pixels, z.max_filter_pixels)?;
    pair("blob_pixels", z.min_blob_pixels, z.max_blob_pixels)?;
    pair("blobs", z.min_blobs, z.max_blobs)
}

/// Apply a partial update: fields left out are kept, `null` clears nullable
/// ones. `num_coords` and `area` follow `coords`, because percent thresholds
/// are relative to the area (GH #43). Nothing is written if any value is
/// invalid.
pub async fn update(
    db: &DatabaseConnection,
    id: u32,
    req: &crate::dto::request::zones::UpdateZoneRequest,
) -> AppResult<Option<ZoneModel>> {
    use sea_orm::{ActiveModelTrait, Set};
    let Some(current) = Zones::find_by_id(id).one(db).await? else {
        return Ok(None);
    };
    let mut z = current.clone();
    if let Some(n) = &req.name {
        z.name = n.clone();
    }
    if let Some(t) = &req.r#type {
        z.r#type = parse_zone_type(t)?;
    }
    if let Some(u) = &req.units {
        z.units = parse_units(u)?;
    }
    if let Some(c) = req.coords.as_ref().or(req.polygon.as_ref()) {
        let (area, count) = polygon(c)?;
        z.coords = c.clone();
        z.area = area;
        z.num_coords = count;
    }
    if let Some(m) = &req.check_method {
        z.check_method = parse_check_method(m)?;
    }
    let dec = |v: Option<f64>| v.and_then(to_decimal);
    if let Some(v) = req.alarm_rgb {
        z.alarm_rgb = v;
    }
    if let Some(v) = req.min_pixel_threshold {
        z.min_pixel_threshold = v;
    }
    if let Some(v) = req.max_pixel_threshold {
        z.max_pixel_threshold = v;
    }
    if let Some(v) = req.min_alarm_pixels {
        z.min_alarm_pixels = dec(v);
    }
    if let Some(v) = req.max_alarm_pixels {
        z.max_alarm_pixels = dec(v);
    }
    if let Some(v) = req.filter_x {
        z.filter_x = v;
    }
    if let Some(v) = req.filter_y {
        z.filter_y = v;
    }
    if let Some(v) = req.min_filter_pixels {
        z.min_filter_pixels = dec(v);
    }
    if let Some(v) = req.max_filter_pixels {
        z.max_filter_pixels = dec(v);
    }
    if let Some(v) = req.min_blob_pixels {
        z.min_blob_pixels = dec(v);
    }
    if let Some(v) = req.max_blob_pixels {
        z.max_blob_pixels = dec(v);
    }
    if let Some(v) = req.min_blobs {
        z.min_blobs = v;
    }
    if let Some(v) = req.max_blobs {
        z.max_blobs = v;
    }
    if let Some(v) = req.overload_frames {
        z.overload_frames = v;
    }
    if let Some(v) = req.extend_alarm_frames {
        z.extend_alarm_frames = v;
    }
    check_ranges(&z)?;
    if z == current {
        return Ok(Some(current));
    }

    let mut active: crate::entity::zones::ActiveModel = current.into();
    active.name = Set(z.name);
    active.r#type = Set(z.r#type);
    active.units = Set(z.units);
    active.num_coords = Set(z.num_coords);
    active.coords = Set(z.coords);
    active.area = Set(z.area);
    active.alarm_rgb = Set(z.alarm_rgb);
    active.check_method = Set(z.check_method);
    active.min_pixel_threshold = Set(z.min_pixel_threshold);
    active.max_pixel_threshold = Set(z.max_pixel_threshold);
    active.min_alarm_pixels = Set(z.min_alarm_pixels);
    active.max_alarm_pixels = Set(z.max_alarm_pixels);
    active.filter_x = Set(z.filter_x);
    active.filter_y = Set(z.filter_y);
    active.min_filter_pixels = Set(z.min_filter_pixels);
    active.max_filter_pixels = Set(z.max_filter_pixels);
    active.min_blob_pixels = Set(z.min_blob_pixels);
    active.max_blob_pixels = Set(z.max_blob_pixels);
    active.min_blobs = Set(z.min_blobs);
    active.max_blobs = Set(z.max_blobs);
    active.overload_frames = Set(z.overload_frames);
    active.extend_alarm_frames = Set(z.extend_alarm_frames);
    Ok(Some(active.update(db).await?))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    use sea_orm::EntityTrait;
    let res = Zones::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}

pub async fn create_for_monitor(
    db: &DatabaseConnection,
    monitor_id: u32,
    req: &crate::dto::request::CreateZoneRequest,
) -> AppResult<ZoneModel> {
    use crate::entity::zones::ActiveModel as AM;
    use sea_orm::{ActiveModelTrait, Set};

    let (area, num_coords) = polygon(&req.coords)?;
    let dec = |v: Option<f64>| v.and_then(to_decimal);
    let model = ZoneModel {
        id: 0,
        monitor_id,
        name: req.name.clone(),
        r#type: parse_zone_type(&req.r#type)?,
        units: parse_units(&req.units)?,
        num_coords,
        coords: req.coords.clone(),
        // Computed, not zero: percent thresholds are relative to it (GH #43).
        area,
        alarm_rgb: req.alarm_rgb,
        check_method: match req.check_method.as_deref() {
            Some(m) => parse_check_method(m)?,
            None => crate::entity::sea_orm_active_enums::CheckMethod::AlarmedPixels,
        },
        min_pixel_threshold: req.min_pixel_threshold,
        max_pixel_threshold: req.max_pixel_threshold,
        min_alarm_pixels: dec(req.min_alarm_pixels),
        max_alarm_pixels: dec(req.max_alarm_pixels),
        filter_x: req.filter_x,
        filter_y: req.filter_y,
        min_filter_pixels: dec(req.min_filter_pixels),
        max_filter_pixels: dec(req.max_filter_pixels),
        min_blob_pixels: dec(req.min_blob_pixels),
        max_blob_pixels: dec(req.max_blob_pixels),
        min_blobs: req.min_blobs,
        max_blobs: req.max_blobs,
        overload_frames: req.overload_frames.unwrap_or(0),
        extend_alarm_frames: req.extend_alarm_frames.unwrap_or(0),
    };
    check_ranges(&model)?;

    let active = AM {
        id: Default::default(),
        monitor_id: Set(model.monitor_id),
        name: Set(model.name),
        r#type: Set(model.r#type),
        units: Set(model.units),
        num_coords: Set(model.num_coords),
        coords: Set(model.coords),
        area: Set(model.area),
        alarm_rgb: Set(model.alarm_rgb),
        check_method: Set(model.check_method),
        min_pixel_threshold: Set(model.min_pixel_threshold),
        max_pixel_threshold: Set(model.max_pixel_threshold),
        min_alarm_pixels: Set(model.min_alarm_pixels),
        max_alarm_pixels: Set(model.max_alarm_pixels),
        filter_x: Set(model.filter_x),
        filter_y: Set(model.filter_y),
        min_filter_pixels: Set(model.min_filter_pixels),
        max_filter_pixels: Set(model.max_filter_pixels),
        min_blob_pixels: Set(model.min_blob_pixels),
        max_blob_pixels: Set(model.max_blob_pixels),
        min_blobs: Set(model.min_blobs),
        max_blobs: Set(model.max_blobs),
        overload_frames: Set(model.overload_frames),
        extend_alarm_frames: Set(model.extend_alarm_frames),
    };
    Ok(active.insert(db).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::sea_orm_active_enums::{CheckMethod, Units, ZoneType};
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    fn mk(id: u32, name: &str, coords: &str) -> ZoneModel {
        ZoneModel {
            id,
            monitor_id: 1,
            name: name.into(),
            r#type: ZoneType::Active,
            units: Units::Pixels,
            num_coords: 4,
            coords: coords.into(),
            area: 0,
            alarm_rgb: None,
            check_method: CheckMethod::AlarmedPixels,
            min_pixel_threshold: None,
            max_pixel_threshold: None,
            min_alarm_pixels: None,
            max_alarm_pixels: None,
            filter_x: None,
            filter_y: None,
            min_filter_pixels: None,
            max_filter_pixels: None,
            min_blob_pixels: None,
            max_blob_pixels: None,
            min_blobs: None,
            max_blobs: None,
            overload_frames: 0,
            extend_alarm_frames: 0,
        }
    }

    #[tokio::test]
    async fn test_update_coords_happy_path() {
        let initial = mk(11, "old", "0,0 10,0 10,10");
        let after = mk(11, "new", "0,0 10,0 10,10");
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ZoneModel, _, _>(vec![vec![initial]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_query_results::<ZoneModel, _, _>(vec![vec![after.clone()]])
            .into_connection();

        let updated = update_coords(&db, 11, Some("new".into()), Some("0,0 10,0 10,10".into()))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.name, "new");
        assert_eq!(updated.coords, "0,0 10,0 10,10");
    }

    #[tokio::test]
    async fn test_delete_by_id_affects_rows() {
        let db_true = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();
        assert!(delete_by_id(&db_true, 1).await.unwrap());

        let db_false = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 0,
            }])
            .into_connection();
        assert!(!delete_by_id(&db_false, 1).await.unwrap());
    }
}
