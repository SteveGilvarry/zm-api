use crate::dto::PaginationParams;
use crate::entity::zone_presets::{Entity as ZonePresets, Model as ZonePresetModel};
use crate::error::AppResult;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<ZonePresetModel>> {
    Ok(ZonePresets::find().all(db).await?)
}

pub async fn find_paginated(
    db: &DatabaseConnection,
    params: &PaginationParams,
) -> AppResult<(Vec<ZonePresetModel>, u64)> {
    let paginator = ZonePresets::find().paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<ZonePresetModel>> {
    Ok(ZonePresets::find_by_id(id).one(db).await?)
}

pub async fn create(
    db: &DatabaseConnection,
    req: &crate::dto::request::CreateZonePresetRequest,
) -> AppResult<ZonePresetModel> {
    use crate::entity::zone_presets::ActiveModel as AM;
    use sea_orm::{ActiveModelTrait, Set};
    fn parse_zone_type(s: &str) -> crate::entity::sea_orm_active_enums::ZoneType {
        use crate::entity::sea_orm_active_enums::ZoneType::*;
        match s.to_lowercase().as_str() {
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
    let am = AM {
        id: Default::default(),
        name: Set(req.name.clone()),
        r#type: Set(parse_zone_type(&req.r#type)),
        units: Set(parse_units(&req.units)),
        check_method: Set(parse_check_method(&req.check_method)),
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
    Ok(am.insert(db).await?)
}

pub async fn update(
    db: &DatabaseConnection,
    id: u32,
    name: Option<String>,
    r#type: Option<String>,
    units: Option<String>,
    check_method: Option<String>,
) -> AppResult<Option<ZonePresetModel>> {
    use sea_orm::{ActiveModelTrait, Set};
    let Some(model) = find_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: crate::entity::zone_presets::ActiveModel = model.into();
    fn parse_zone_type(s: &str) -> crate::entity::sea_orm_active_enums::ZoneType {
        use crate::entity::sea_orm_active_enums::ZoneType::*;
        match s.to_lowercase().as_str() {
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
    if let Some(v) = name {
        am.name = Set(v);
    }
    if let Some(v) = r#type {
        am.r#type = Set(parse_zone_type(&v));
    }
    if let Some(v) = units {
        am.units = Set(parse_units(&v));
    }
    if let Some(v) = check_method {
        am.check_method = Set(parse_check_method(&v));
    }
    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    use sea_orm::EntityTrait;
    let res = ZonePresets::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::sea_orm_active_enums::{CheckMethod, Units, ZoneType};
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    fn mk(id: u32, name: &str) -> ZonePresetModel {
        ZonePresetModel {
            id,
            name: name.into(),
            r#type: ZoneType::Active,
            units: Units::Pixels,
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
    async fn test_update_happy_path() {
        let initial = mk(8, "old");
        let after = ZonePresetModel {
            name: "new".into(),
            r#type: ZoneType::Inclusive,
            ..initial.clone()
        };
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ZonePresetModel, _, _>(vec![vec![initial]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_query_results::<ZonePresetModel, _, _>(vec![vec![after.clone()]])
            .into_connection();

        let updated = update(
            &db,
            8,
            Some("new".into()),
            Some("inclusive".into()),
            None,
            None,
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(updated.name, "new");
        assert_eq!(updated.r#type, ZoneType::Inclusive);
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
