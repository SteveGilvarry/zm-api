use crate::dto::request::config::UpdateConfigRequest;
use crate::dto::response::config::ConfigResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult};
use crate::repo;
use crate::server::state::AppState;

fn to_response(m: &crate::entity::config::Model) -> ConfigResponse {
    ConfigResponse {
        id: m.id,
        name: m.name.clone(),
        value: m.value.clone(),
        r#type: m.r#type.clone(),
        default_value: m.default_value.clone(),
        hint: m.hint.clone(),
        pattern: m.pattern.clone(),
        format: m.format.clone(),
        prompt: m.prompt.clone(),
        help: m.help.clone(),
        category: m.category.clone(),
        readonly: m.readonly,
        private: m.private,
        system: m.system,
    }
}

pub async fn list_all(state: &AppState) -> AppResult<Vec<ConfigResponse>> {
    let items = repo::config::list_all(state.db()).await?;
    Ok(items.iter().map(to_response).collect())
}

pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
) -> AppResult<PaginatedResponse<ConfigResponse>> {
    let (items, total) = repo::config::find_paginated(state.db(), params).await?;
    let responses: Vec<ConfigResponse> = items.iter().map(to_response).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_by_name(state: &AppState, name: &str) -> AppResult<ConfigResponse> {
    let item = repo::config::find_by_name(state.db(), name).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(crate::error::Resource {
            details: vec![("name".into(), name.to_string())],
            resource_type: crate::error::ResourceType::Config,
        })
    })?;
    Ok(to_response(&item))
}

pub async fn update_value(
    state: &AppState,
    name: &str,
    req: UpdateConfigRequest,
) -> AppResult<ConfigResponse> {
    // Load and enforce readonly
    let existing = repo::config::find_by_name(state.db(), name).await?;
    let existing = existing.ok_or_else(|| {
        AppError::NotFoundError(crate::error::Resource {
            details: vec![("name".into(), name.to_string())],
            resource_type: crate::error::ResourceType::Config,
        })
    })?;
    if existing.readonly != 0 {
        return Err(AppError::PermissionDeniedError(
            "Config is read-only".into(),
        ));
    }

    let updated = repo::config::update_value(state.db(), name, &req.value).await?;
    let updated =
        updated.ok_or_else(|| AppError::InternalServerError("Failed to update config".into()))?;
    Ok(to_response(&updated))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::config::Model as ConfigModel;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    fn mk_config(name: &str, value: &str, readonly: u8) -> ConfigModel {
        ConfigModel {
            id: 1,
            name: name.into(),
            value: value.into(),
            r#type: "string".into(),
            default_value: None,
            hint: None,
            pattern: None,
            format: None,
            prompt: None,
            help: None,
            category: "General".into(),
            readonly,
            private: 0,
            system: 0,
            requires: None,
        }
    }

    #[tokio::test]
    async fn test_list_all_and_get_by_name() {
        let db_list = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ConfigModel, _, _>(vec![vec![
                mk_config("A", "1", 0),
                mk_config("B", "2", 0),
            ]])
            .into_connection();
        let state_list = AppState::for_test_with_db(db_list);
        assert_eq!(list_all(&state_list).await.unwrap().len(), 2);

        let db_get = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ConfigModel, _, _>(vec![vec![mk_config("X", "42", 0)]])
            .into_connection();
        let state_get = AppState::for_test_with_db(db_get);
        assert_eq!(get_by_name(&state_get, "X").await.unwrap().value, "42");

        let empty: Vec<ConfigModel> = vec![];
        let db_none = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ConfigModel, _, _>(vec![empty])
            .into_connection();
        let state_none = AppState::for_test_with_db(db_none);
        assert!(matches!(
            get_by_name(&state_none, "M").await.err().unwrap(),
            AppError::NotFoundError(_)
        ));
    }

    #[tokio::test]
    async fn test_update_value_ok_and_readonly() {
        use crate::dto::request::config::UpdateConfigRequest;
        // Happy path
        let db_ok = MockDatabase::new(DatabaseBackend::MySql)
            // existing (service-level read)
            .append_query_results::<ConfigModel, _, _>(vec![vec![mk_config("Key", "old", 0)]])
            // existing again (repo::config::find_by_name inside update_value)
            .append_query_results::<ConfigModel, _, _>(vec![vec![mk_config("Key", "old", 0)]])
            // update exec
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            // updated row returned by update
            .append_query_results::<ConfigModel, _, _>(vec![vec![mk_config("Key", "new", 0)]])
            .into_connection();
        let state_ok = AppState::for_test_with_db(db_ok);
        let out = update_value(
            &state_ok,
            "Key",
            UpdateConfigRequest {
                value: "new".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(out.value, "new");

        // Read-only guard
        let db_ro = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ConfigModel, _, _>(vec![vec![mk_config("Key", "old", 1)]])
            .into_connection();
        let state_ro = AppState::for_test_with_db(db_ro);
        let err = update_value(&state_ro, "Key", UpdateConfigRequest { value: "x".into() })
            .await
            .err()
            .unwrap();
        assert!(matches!(err, AppError::PermissionDeniedError(_)));
    }
}
