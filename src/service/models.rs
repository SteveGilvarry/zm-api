use crate::dto::response::ModelResponse;
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(
    state: &AppState,
    manufacturer_id: Option<u32>,
) -> AppResult<Vec<ModelResponse>> {
    let items = repo::models::find_all(state.db(), manufacturer_id).await?;
    Ok(items.iter().map(ModelResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<ModelResponse> {
    let item = repo::models::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(ModelResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    req: crate::dto::request::CreateModelRequest,
) -> AppResult<ModelResponse> {
    let model = repo::models::create(state.db(), &req).await?;
    Ok(ModelResponse::from(&model))
}

pub async fn update(
    state: &AppState,
    id: u32,
    name: Option<String>,
    manufacturer_id: Option<i32>,
) -> AppResult<ModelResponse> {
    let updated = repo::models::update(state.db(), id, name, manufacturer_id).await?;
    let updated = updated.ok_or_else(|| {
        crate::error::AppError::NotFoundError(crate::error::Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: crate::error::ResourceType::Message,
        })
    })?;
    Ok(ModelResponse::from(&updated))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::models::delete_by_id(state.db(), id).await?;
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
    use crate::entity::models::Model as ModelModel;
    use crate::error::AppError;
    use crate::server::state::AppState;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let empty: Vec<ModelModel> = vec![];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ModelModel, _, _>(vec![empty])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let err = get_by_id(&state, 1).await.err().expect("should err");
        matches!(err, AppError::NotFoundError(_));
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let empty: Vec<ModelModel> = vec![];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ModelModel, _, _>(vec![empty])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let err = update(&state, 1, Some("x".into()), Some(2))
            .await
            .err()
            .expect("should err");
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
        let err = delete(&state, 1).await.err().expect("should err");
        matches!(err, AppError::NotFoundError(_));
    }

    fn mk(id: u32, name: &str, man: Option<i32>) -> ModelModel {
        ModelModel {
            id,
            name: name.into(),
            manufacturer_id: man,
        }
    }

    #[tokio::test]
    async fn test_list_all_ok() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ModelModel, _, _>(vec![vec![
                mk(1, "m1", None),
                mk(2, "m2", Some(3)),
            ]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let out = list_all(&state, None).await.unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].name, "m1");
    }

    #[tokio::test]
    async fn test_get_by_id_ok() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ModelModel, _, _>(vec![vec![mk(9, "cam", None)]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let out = get_by_id(&state, 9).await.unwrap();
        assert_eq!(out.id, 9);
        assert_eq!(out.name, "cam");
    }

    #[tokio::test]
    async fn test_update_ok() {
        let initial = mk(3, "old", None);
        let after = mk(3, "new", Some(7));
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ModelModel, _, _>(vec![vec![initial]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_query_results::<ModelModel, _, _>(vec![vec![after.clone()]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let out = update(&state, 3, Some("new".into()), Some(7))
            .await
            .unwrap();
        assert_eq!(out.name, "new");
        assert_eq!(out.manufacturer_id, Some(7));
    }

    #[tokio::test]
    async fn test_create_ok() {
        use crate::dto::request::models::CreateModelRequest;
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 55,
                rows_affected: 1,
            }])
            .append_query_results::<ModelModel, _, _>(vec![vec![mk(55, "nm", Some(2))]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let req = CreateModelRequest {
            name: "nm".into(),
            manufacturer_id: Some(2),
        };
        let out = create(&state, req).await.unwrap();
        assert_eq!(out.name, "nm");
    }
}
