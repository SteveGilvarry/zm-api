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

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<ZoneModel>> {
    let zone = Zones::find_by_id(id).one(db).await?;
    Ok(zone)
}

pub async fn update_coords(
    db: &DatabaseConnection,
    id: u32,
    name: Option<String>,
    coords: Option<String>,
) -> AppResult<Option<ZoneModel>> {
    use sea_orm::ActiveModelTrait;
    use sea_orm::Set;
    if let Some(model) = Zones::find_by_id(id).one(db).await? {
        let mut active: crate::entity::zones::ActiveModel = model.into();
        if let Some(n) = name {
            active.name = Set(n);
        }
        if let Some(c) = coords {
            active.coords = Set(c);
        }
        let updated = active.update(db).await?;
        Ok(Some(updated))
    } else {
        Ok(None)
    }
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
    fn parse_zone_type(s: &str) -> crate::entity::sea_orm_active_enums::ZoneType {
        use crate::entity::sea_orm_active_enums::ZoneType::*;
        match s.to_lowercase().as_str() {
            "active" => Active,
            "inclusive" => Inclusive,
            "exclusive" => Exclusive,
            "preclusive" => Preclusive,
            "inactive" => Inactive,
            "privacy" => Privacy,
            _ => Active,
        }
    }
    fn parse_units(s: &str) -> crate::entity::sea_orm_active_enums::Units {
        use crate::entity::sea_orm_active_enums::Units::*;
        match s.to_lowercase().as_str() {
            "pixels" => Pixels,
            "percent" => Percent,
            _ => Pixels,
        }
    }
    fn parse_check_method(s: &str) -> crate::entity::sea_orm_active_enums::CheckMethod {
        use crate::entity::sea_orm_active_enums::CheckMethod::*;
        match s.to_lowercase().as_str() {
            "filteredpixels" => FilteredPixels,
            "blobs" => Blobs,
            _ => AlarmedPixels,
        }
    }
    let model = AM {
        id: Default::default(),
        monitor_id: Set(monitor_id),
        name: Set(req.name.clone()),
        r#type: Set(parse_zone_type(&req.r#type)),
        units: Set(parse_units(&req.units)),
        num_coords: Set(req.num_coords),
        coords: Set(req.coords.clone()),
        area: Set(0),
        alarm_rgb: Set(None),
        check_method: Set(req
            .check_method
            .as_deref()
            .map(parse_check_method)
            .unwrap_or(crate::entity::sea_orm_active_enums::CheckMethod::AlarmedPixels)),
        min_pixel_threshold: Set(None),
        max_pixel_threshold: Set(None),
        min_alarm_pixels: Set(None),
        max_alarm_pixels: Set(None),
        filter_x: Set(None),
        filter_y: Set(None),
        min_filter_pixels: Set(None),
        max_filter_pixels: Set(None),
        min_blob_pixels: Set(None),
        max_blob_pixels: Set(None),
        min_blobs: Set(None),
        max_blobs: Set(None),
        overload_frames: Set(0),
        extend_alarm_frames: Set(0),
    };
    Ok(model.insert(db).await?)
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
        let initial = mk(11, "old", "0,0 1,1");
        let after = mk(11, "new", "2,2 3,3");
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ZoneModel, _, _>(vec![vec![initial]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_query_results::<ZoneModel, _, _>(vec![vec![after.clone()]])
            .into_connection();

        let updated = update_coords(&db, 11, Some("new".into()), Some("2,2 3,3".into()))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.name, "new");
        assert_eq!(updated.coords, "2,2 3,3");
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
