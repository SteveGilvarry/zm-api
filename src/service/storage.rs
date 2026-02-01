use crate::dto::response::StorageResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState) -> AppResult<Vec<StorageResponse>> {
    let items = repo::storage::find_all(state.db()).await?;
    Ok(items.iter().map(StorageResponse::from).collect())
}

pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
) -> AppResult<PaginatedResponse<StorageResponse>> {
    let (items, total) = repo::storage::find_paginated(state.db(), params).await?;
    let responses: Vec<StorageResponse> = items.iter().map(StorageResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_by_id(state: &AppState, id: u16) -> AppResult<StorageResponse> {
    let item = repo::storage::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(StorageResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    req: crate::dto::request::CreateStorageRequest,
) -> AppResult<StorageResponse> {
    let model = repo::storage::create(state.db(), &req).await?;
    Ok(StorageResponse::from(&model))
}

#[allow(clippy::too_many_arguments)]
pub async fn update(
    state: &AppState,
    id: u16,
    name: Option<String>,
    path: Option<String>,
    r#type: Option<String>,
    enabled: Option<i8>,
    scheme: Option<String>,
    server_id: Option<u32>,
    url: Option<String>,
) -> AppResult<StorageResponse> {
    let updated = repo::storage::update(
        state.db(),
        id,
        name,
        path,
        r#type,
        enabled,
        scheme,
        server_id,
        url,
    )
    .await?;
    let updated = updated.ok_or_else(|| {
        crate::error::AppError::NotFoundError(crate::error::Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: crate::error::ResourceType::Message,
        })
    })?;
    Ok(StorageResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u16) -> AppResult<()> {
    let ok = repo::storage::delete_by_id(state.db(), id).await?;
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
    use crate::entity::storage::Model as StorageModel;
    use crate::error::AppError;
    use crate::server::state::AppState;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let empty: Vec<StorageModel> = vec![];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<StorageModel, _, _>(vec![empty])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let err = get_by_id(&state, 9).await.expect_err("should err");
        matches!(err, AppError::NotFoundError(_));
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let empty: Vec<StorageModel> = vec![];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<StorageModel, _, _>(vec![empty])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let err = update(
            &state,
            1,
            Some("n".into()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .expect_err("should err");
        matches!(err, AppError::NotFoundError(_));
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 0,
            }])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let err = delete(&state, 1).await.expect_err("should err");
        matches!(err, AppError::NotFoundError(_));
    }

    fn mk(id: u16, name: &str) -> StorageModel {
        use crate::entity::sea_orm_active_enums::{Scheme, StorageType};
        StorageModel {
            id,
            path: "/tmp".into(),
            name: name.into(),
            r#type: StorageType::Local,
            url: None,
            disk_space: None,
            scheme: Scheme::Deep,
            server_id: None,
            do_delete: 0,
            enabled: 1,
        }
    }

    #[tokio::test]
    async fn test_list_all_ok() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<StorageModel, _, _>(vec![vec![mk(1, "s1"), mk(2, "s2")]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let out = list_all(&state).await.unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].name, "s1");
    }

    #[tokio::test]
    async fn test_get_by_id_ok() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<StorageModel, _, _>(vec![vec![mk(5, "disk")]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let out = get_by_id(&state, 5).await.unwrap();
        assert_eq!(out.id, 5);
        assert_eq!(out.name, "disk");
    }

    #[tokio::test]
    async fn test_update_ok() {
        let initial = mk(7, "old");
        let after = StorageModel {
            name: "new".into(),
            ..initial.clone()
        };
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<StorageModel, _, _>(vec![vec![initial]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_query_results::<StorageModel, _, _>(vec![vec![after.clone()]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let out = update(
            &state,
            7,
            Some("new".into()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
        assert_eq!(out.name, "new");
    }

    #[tokio::test]
    async fn test_create_ok() {
        use crate::dto::request::storage::CreateStorageRequest;
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 44,
                rows_affected: 1,
            }])
            .append_query_results::<StorageModel, _, _>(vec![vec![mk(44, "new")]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let req = CreateStorageRequest {
            path: "/tmp".into(),
            name: "new".into(),
            r#type: "local".into(),
            url: None,
            enabled: 1,
            scheme: None,
            server_id: None,
        };
        let out = create(&state, req).await.unwrap();
        assert_eq!(out.name, "new");
    }

    #[tokio::test]
    async fn test_delete_ok() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        assert!(delete(&state, 1).await.is_ok());
    }
}
