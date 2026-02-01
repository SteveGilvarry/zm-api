use crate::dto::response::ZonePresetResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState) -> AppResult<Vec<ZonePresetResponse>> {
    let items = repo::zone_presets::find_all(state.db()).await?;
    Ok(items.iter().map(ZonePresetResponse::from).collect())
}

pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
) -> AppResult<PaginatedResponse<ZonePresetResponse>> {
    let (items, total) = repo::zone_presets::find_paginated(state.db(), params).await?;
    let responses: Vec<ZonePresetResponse> = items.iter().map(ZonePresetResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<ZonePresetResponse> {
    let item = repo::zone_presets::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(ZonePresetResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    req: crate::dto::request::CreateZonePresetRequest,
) -> AppResult<ZonePresetResponse> {
    let model = repo::zone_presets::create(state.db(), &req).await?;
    Ok(ZonePresetResponse::from(&model))
}

pub async fn update(
    state: &AppState,
    id: u32,
    name: Option<String>,
    r#type: Option<String>,
    units: Option<String>,
    check_method: Option<String>,
) -> AppResult<ZonePresetResponse> {
    let updated =
        repo::zone_presets::update(state.db(), id, name, r#type, units, check_method).await?;
    let updated = updated.ok_or_else(|| {
        crate::error::AppError::NotFoundError(crate::error::Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: crate::error::ResourceType::Message,
        })
    })?;
    Ok(ZonePresetResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::zone_presets::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(crate::error::AppError::NotFoundError(
            crate::error::Resource {
                details: vec![("id".into(), id.to_string())],
                resource_type: crate::error::ResourceType::Message,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::sea_orm_active_enums::{CheckMethod, Units, ZoneType};
    use crate::entity::zone_presets::Model as ZpModel;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    fn mk(id: u32, name: &str) -> ZpModel {
        ZpModel {
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
    async fn test_list_get_ok() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ZpModel, _, _>(vec![vec![mk(1, "a"), mk(2, "b")]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        assert_eq!(list_all(&state).await.unwrap().len(), 2);

        let db2 = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ZpModel, _, _>(vec![vec![mk(9, "z")]])
            .into_connection();
        let state2 = AppState::for_test_with_db(db2);
        assert_eq!(get_by_id(&state2, 9).await.unwrap().name, "z");
    }

    #[tokio::test]
    async fn test_update_create_delete_paths() {
        use crate::dto::request::zone_presets::CreateZonePresetRequest;
        let initial = mk(5, "old");
        let after = ZpModel {
            name: "new".into(),
            r#type: ZoneType::Inclusive,
            ..initial.clone()
        };
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ZpModel, _, _>(vec![vec![initial]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_query_results::<ZpModel, _, _>(vec![vec![after.clone()]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let out = update(
            &state,
            5,
            Some("new".into()),
            Some("inclusive".into()),
            None,
            None,
        )
        .await
        .unwrap();
        assert_eq!(out.name, "new");
        assert_eq!(out.r#type, "Inclusive");

        let db_create = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 44,
                rows_affected: 1,
            }])
            .append_query_results::<ZpModel, _, _>(vec![vec![mk(44, "np")]])
            .into_connection();
        let state_create = AppState::for_test_with_db(db_create);
        let req = CreateZonePresetRequest {
            name: "np".into(),
            r#type: "active".into(),
            units: "pixels".into(),
            check_method: "alarmedpixels".into(),
        };
        assert_eq!(create(&state_create, req).await.unwrap().name, "np");

        let db_del_ok = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();
        let state_del_ok = AppState::for_test_with_db(db_del_ok);
        assert!(delete(&state_del_ok, 1).await.is_ok());

        let empty: Vec<ZpModel> = vec![];
        let db_none = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ZpModel, _, _>(vec![empty])
            .into_connection();
        let state_none = AppState::for_test_with_db(db_none);
        assert!(matches!(
            get_by_id(&state_none, 1).await.err().unwrap(),
            AppError::NotFoundError(_)
        ));

        let db_del_none = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 0,
            }])
            .into_connection();
        let state_del_none = AppState::for_test_with_db(db_del_none);
        assert!(matches!(
            delete(&state_del_none, 1).await.err().unwrap(),
            AppError::NotFoundError(_)
        ));
    }
}
