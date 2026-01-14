use crate::dto::response::FilterResponse;
use crate::error::{AppError, AppResult};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState) -> AppResult<Vec<FilterResponse>> {
    let items = repo::filters::find_all(state.db()).await?;
    Ok(items.iter().map(FilterResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<FilterResponse> {
    let item = repo::filters::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(crate::error::Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: crate::error::ResourceType::Message,
        })
    })?;
    Ok(FilterResponse::from(&item))
}

pub async fn update(
    state: &AppState,
    id: u32,
    name: Option<String>,
    query_json: Option<String>,
) -> AppResult<FilterResponse> {
    let item = repo::filters::update(state.db(), id, name, query_json).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(crate::error::Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: crate::error::ResourceType::Message,
        })
    })?;
    Ok(FilterResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    req: crate::dto::request::CreateFilterRequest,
) -> AppResult<FilterResponse> {
    let model = repo::filters::create(state.db(), &req).await?;
    Ok(FilterResponse::from(&model))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::filters::delete_by_id(state.db(), id).await?;
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
    use crate::entity::filters::Model as FilterModel;
    use crate::error::AppError;
    use crate::server::state::AppState;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    fn mk_filter(id: u32, name: &str) -> FilterModel {
        use crate::entity::sea_orm_active_enums::EmailFormat;
        FilterModel {
            id,
            name: name.into(),
            user_id: Some(1),
            execute_interval: 0,
            query_json: "{}".into(),
            auto_archive: 0,
            auto_unarchive: 0,
            auto_video: 0,
            auto_upload: 0,
            auto_email: 0,
            email_to: None,
            email_subject: None,
            email_body: None,
            email_format: EmailFormat::Summary,
            auto_message: 0,
            auto_execute: 0,
            auto_execute_cmd: None,
            auto_delete: 0,
            auto_move: 0,
            auto_copy: 0,
            auto_copy_to: 0,
            auto_move_to: 0,
            update_disk_space: 0,
            background: 0,
            concurrent: 0,
            lock_rows: 0,
        }
    }

    #[tokio::test]
    async fn test_get_by_id_ok() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<FilterModel, _, _>(vec![vec![mk_filter(1, "f1")]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let resp = get_by_id(&state, 1).await.unwrap();
        assert_eq!(resp.id, 1);
        assert_eq!(resp.name, "f1");
    }

    #[tokio::test]
    async fn test_update_ok() {
        let initial = mk_filter(2, "old");
        let updated = FilterModel {
            name: "new".into(),
            query_json: "{\"k\":\"v\"}".into(),
            ..initial.clone()
        };
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<FilterModel, _, _>(vec![vec![initial]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_query_results::<FilterModel, _, _>(vec![vec![updated.clone()]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let resp = update(&state, 2, Some("new".into()), Some("{\"k\":\"v\"}".into()))
            .await
            .unwrap();
        assert_eq!(resp.name, "new");
        assert_eq!(resp.query_json, "{\"k\":\"v\"}");
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

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let empty: Vec<FilterModel> = vec![];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<FilterModel, _, _>(vec![empty])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let err = get_by_id(&state, 99).await.expect_err("should error");
        matches!(err, AppError::NotFoundError(_));
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let empty: Vec<FilterModel> = vec![];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<FilterModel, _, _>(vec![empty])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let err = update(&state, 1, Some("x".into()), None)
            .await
            .expect_err("should error");
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
        let err = delete(&state, 1).await.expect_err("should error");
        matches!(err, AppError::NotFoundError(_));
    }
}
