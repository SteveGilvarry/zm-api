use crate::dto::response::ZoneResponse;
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_by_monitor(state: &AppState, monitor_id: u32) -> AppResult<Vec<ZoneResponse>> {
    let zones = repo::zones::find_by_monitor(state.db(), monitor_id).await?;
    Ok(zones.iter().map(ZoneResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<ZoneResponse> {
    let zone = repo::zones::find_by_id(state.db(), id).await?;
    let zone = zone.ok_or_else(|| AppError::NotFoundError(Resource{details: vec![("id".into(), id.to_string())], resource_type: ResourceType::Monitor}))?; // ResourceType Zone not defined; using Monitor placeholder
    Ok(ZoneResponse::from(&zone))
}

pub async fn update(state: &AppState, id: u32, name: Option<String>, coords: Option<String>) -> AppResult<ZoneResponse> {
    let updated = repo::zones::update_coords(state.db(), id, name, coords).await?;
    let updated = updated.ok_or_else(|| AppError::NotFoundError(Resource{details: vec![("id".into(), id.to_string())], resource_type: ResourceType::Monitor}))?;
    Ok(ZoneResponse::from(&updated))
}

pub async fn create(state: &AppState, monitor_id: u32, req: crate::dto::request::CreateZoneRequest) -> AppResult<ZoneResponse> {
    let model = repo::zones::create_for_monitor(state.db(), monitor_id, &req).await?;
    Ok(ZoneResponse::from(&model))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::zones::delete_by_id(state.db(), id).await?;
    if ok { Ok(()) } else { Err(crate::error::AppError::NotFoundError(crate::error::Resource{details: vec![("id".into(), id.to_string())], resource_type: crate::error::ResourceType::Monitor})) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
    use crate::entity::zones::Model as ZoneModel;
    use crate::entity::sea_orm_active_enums::{ZoneType, Units, CheckMethod};

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
    async fn test_list_by_monitor_and_get() {
        let db_list = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ZoneModel, _, _>(vec![vec![mk(1, "z1", "0,0 1,1"), mk(2, "z2", "2,2 3,3")]])
            .into_connection();
        let state_list = AppState::for_test_with_db(db_list);
        assert_eq!(list_by_monitor(&state_list, 1).await.unwrap().len(), 2);

        let db_get = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ZoneModel, _, _>(vec![vec![mk(9, "z", "0,0 1,1")]])
            .into_connection();
        let state_get = AppState::for_test_with_db(db_get);
        assert_eq!(get_by_id(&state_get, 9).await.unwrap().id, 9);
    }

    #[tokio::test]
    async fn test_update_create_delete_paths() {
        use crate::dto::request::zones::CreateZoneRequest;
        let initial = mk(5, "old", "0,0 1,1");
        let after = mk(5, "new", "2,2 3,3");
        let db_upd = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ZoneModel, _, _>(vec![vec![initial]])
            .append_exec_results(vec![MockExecResult { last_insert_id: 0, rows_affected: 1 }])
            .append_query_results::<ZoneModel, _, _>(vec![vec![after.clone()]])
            .into_connection();
        let state_upd = AppState::for_test_with_db(db_upd);
        let out = update(&state_upd, 5, Some("new".into()), Some("2,2 3,3".into())).await.unwrap();
        assert_eq!(out.name, "new");
        assert_eq!(out.coords, "2,2 3,3");

        let db_create = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult { last_insert_id: 33, rows_affected: 1 }])
            .append_query_results::<ZoneModel, _, _>(vec![vec![mk(33, "nz", "0,0 1,1")]])
            .into_connection();
        let state_create = AppState::for_test_with_db(db_create);
        let req = CreateZoneRequest { name: "nz".into(), r#type: "active".into(), units: "pixels".into(), coords: "0,0 1,1".into(), num_coords: 4, check_method: None };
        assert_eq!(create(&state_create, 1, req).await.unwrap().name, "nz");

        let db_del_ok = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult { last_insert_id: 0, rows_affected: 1 }])
            .into_connection();
        let state_del_ok = AppState::for_test_with_db(db_del_ok);
        assert!(delete(&state_del_ok, 1).await.is_ok());

        let empty: Vec<ZoneModel> = vec![];
        let db_none = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ZoneModel, _, _>(vec![empty])
            .into_connection();
        let state_none = AppState::for_test_with_db(db_none);
        assert!(matches!(get_by_id(&state_none, 1).await.err().unwrap(), AppError::NotFoundError(_)));

        let db_del_none = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult { last_insert_id: 0, rows_affected: 0 }])
            .into_connection();
        let state_del_none = AppState::for_test_with_db(db_del_none);
        assert!(matches!(delete(&state_del_none, 1).await.err().unwrap(), AppError::NotFoundError(_)));
    }
}
