use crate::dto::request::config::{ConfigQueryParams, UpdateConfigRequest};
use crate::dto::response::config::{CategoryCountResponse, ConfigResponse};
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

pub async fn list_filtered(
    state: &AppState,
    params: &ConfigQueryParams,
) -> AppResult<PaginatedResponse<ConfigResponse>> {
    let (items, total) = repo::config::find_filtered(state.db(), params).await?;
    let responses: Vec<ConfigResponse> = items.iter().map(to_response).collect();
    Ok(PaginatedResponse::new(
        responses,
        total,
        params.page(),
        params.page_size(),
    ))
}

pub async fn list_categories(state: &AppState) -> AppResult<Vec<CategoryCountResponse>> {
    repo::config::list_categories(state.db()).await
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
    use std::collections::BTreeMap;

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

    fn mk_config_cat(id: u16, name: &str, category: &str) -> ConfigModel {
        ConfigModel {
            id,
            name: name.into(),
            value: "v".into(),
            r#type: "string".into(),
            default_value: None,
            hint: None,
            pattern: None,
            format: None,
            prompt: None,
            help: None,
            category: category.into(),
            readonly: 0,
            private: 0,
            system: 0,
            requires: None,
        }
    }

    #[tokio::test]
    async fn test_list_filtered_with_category() {
        let items = vec![mk_config_cat(1, "ZM_OPT_X", "System")];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![vec![BTreeMap::from([(
                "num_items".to_string(),
                sea_orm::Value::Int(Some(1)),
            )])]])
            .append_query_results::<ConfigModel, _, _>(vec![items])
            .into_connection();
        let state = AppState::for_test_with_db(db);

        let params = ConfigQueryParams {
            category: Some("System".into()),
            ..Default::default()
        };
        let result = list_filtered(&state, &params).await.unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.items[0].category, "System");
    }

    #[tokio::test]
    async fn test_list_categories_ok() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![vec![
                BTreeMap::from([
                    (
                        "category".to_string(),
                        sea_orm::Value::String(Some(Box::new("Network".to_string()))),
                    ),
                    ("count".to_string(), sea_orm::Value::BigInt(Some(5))),
                ]),
                BTreeMap::from([
                    (
                        "category".to_string(),
                        sea_orm::Value::String(Some(Box::new("System".to_string()))),
                    ),
                    ("count".to_string(), sea_orm::Value::BigInt(Some(10))),
                ]),
            ]])
            .into_connection();
        let state = AppState::for_test_with_db(db);

        let cats = list_categories(&state).await.unwrap();
        assert_eq!(cats.len(), 2);
        assert_eq!(cats[0].category, "Network");
        assert_eq!(cats[1].count, 10);
    }
}
