use crate::dto::response::ManufacturerResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState) -> AppResult<Vec<ManufacturerResponse>> {
    let items = repo::manufacturers::find_all(state.db()).await?;
    Ok(items.iter().map(ManufacturerResponse::from).collect())
}

pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
) -> AppResult<PaginatedResponse<ManufacturerResponse>> {
    let (items, total) = repo::manufacturers::find_paginated(state.db(), params).await?;
    let responses: Vec<ManufacturerResponse> =
        items.iter().map(ManufacturerResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<ManufacturerResponse> {
    let item = repo::manufacturers::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(ManufacturerResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    req: crate::dto::request::CreateManufacturerRequest,
) -> AppResult<ManufacturerResponse> {
    let model = repo::manufacturers::create(state.db(), &req).await?;
    Ok(ManufacturerResponse::from(&model))
}

pub async fn update(
    state: &AppState,
    id: u32,
    name: Option<String>,
) -> AppResult<ManufacturerResponse> {
    let updated = repo::manufacturers::update(state.db(), id, name).await?;
    let updated = updated.ok_or_else(|| {
        crate::error::AppError::NotFoundError(crate::error::Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: crate::error::ResourceType::Message,
        })
    })?;
    Ok(ManufacturerResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::manufacturers::delete_by_id(state.db(), id).await?;
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
    use crate::entity::manufacturers::Model as ManModel;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    fn mk(id: u32, name: &str) -> ManModel {
        ManModel {
            id,
            name: name.into(),
        }
    }

    #[tokio::test]
    async fn test_list_and_get_ok() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ManModel, _, _>(vec![vec![mk(1, "m1"), mk(2, "m2")]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        assert_eq!(list_all(&state).await.unwrap().len(), 2);

        let db2 = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ManModel, _, _>(vec![vec![mk(9, "sony")]])
            .into_connection();
        let state2 = AppState::for_test_with_db(db2);
        assert_eq!(get_by_id(&state2, 9).await.unwrap().name, "sony");
    }

    #[tokio::test]
    async fn test_get_update_delete_not_found() {
        let empty: Vec<ManModel> = vec![];
        // get_by_id on empty
        let db_none_get = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ManModel, _, _>(vec![empty.clone()])
            .into_connection();
        let state_none_get = AppState::for_test_with_db(db_none_get);
        assert!(matches!(
            get_by_id(&state_none_get, 1).await.err().unwrap(),
            AppError::NotFoundError(_)
        ));
        // update on empty
        let db_none_upd = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ManModel, _, _>(vec![empty])
            .into_connection();
        let state_none_upd = AppState::for_test_with_db(db_none_upd);
        assert!(matches!(
            update(&state_none_upd, 1, Some("x".into()))
                .await
                .err()
                .unwrap(),
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

    #[tokio::test]
    async fn test_update_create_delete_ok() {
        use crate::dto::request::manufacturers::CreateManufacturerRequest;
        let initial = mk(3, "old");
        let after = mk(3, "new");
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ManModel, _, _>(vec![vec![initial]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_query_results::<ManModel, _, _>(vec![vec![after.clone()]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        assert_eq!(
            update(&state, 3, Some("new".into())).await.unwrap().name,
            "new"
        );

        let db_create = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 10,
                rows_affected: 1,
            }])
            // some drivers perform a follow-up SELECT: provide created row
            .append_query_results::<ManModel, _, _>(vec![vec![mk(10, "brand")]])
            .into_connection();
        let state_create = AppState::for_test_with_db(db_create);
        let req = CreateManufacturerRequest {
            name: "brand".into(),
        };
        assert_eq!(create(&state_create, req).await.unwrap().name, "brand");

        let db_del_ok = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();
        let state_del_ok = AppState::for_test_with_db(db_del_ok);
        assert!(delete(&state_del_ok, 1).await.is_ok());
    }
}
