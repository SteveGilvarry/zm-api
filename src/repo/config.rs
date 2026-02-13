use crate::dto::request::config::ConfigQueryParams;
use crate::dto::response::config::CategoryCountResponse;
use crate::dto::PaginationParams;
use crate::entity::config;
use crate::entity::config::{Column, Entity as Config};
use crate::error::AppResult;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};

// Constants for config keys
pub const ZM_VERSION_KEY: &str = "ZM_DYN_CURR_VERSION";
pub const ZM_DB_VERSION_KEY: &str = "ZM_DYN_DB_VERSION";

/// Fetch a specific config value by name
#[tracing::instrument(skip_all)]
pub async fn get_config_value(db: &DatabaseConnection, name: &str) -> AppResult<Option<String>> {
    let config = Config::find().filter(Column::Name.eq(name)).one(db).await?;

    Ok(config.map(|c| c.value))
}

/// Get the current ZoneMinder version
#[tracing::instrument(skip_all)]
pub async fn get_zm_version(db: &DatabaseConnection) -> AppResult<String> {
    match get_config_value(db, ZM_VERSION_KEY).await? {
        Some(version) => Ok(version),
        None => Ok("Unknown".to_string()), // Default if version not found
    }
}

/// Get the ZoneMinder database version
#[tracing::instrument(skip_all)]
pub async fn get_zm_db_version(db: &DatabaseConnection) -> AppResult<String> {
    match get_config_value(db, ZM_DB_VERSION_KEY).await? {
        Some(version) => Ok(version),
        None => Ok("Unknown".to_string()), // Default if version not found
    }
}

/// List all config entries
#[tracing::instrument(skip_all)]
pub async fn list_all(db: &DatabaseConnection) -> AppResult<Vec<config::Model>> {
    let items = Config::find().all(db).await?;
    Ok(items)
}

/// List config entries with pagination
#[tracing::instrument(skip_all)]
pub async fn find_paginated(
    db: &DatabaseConnection,
    params: &PaginationParams,
) -> AppResult<(Vec<config::Model>, u64)> {
    let paginator = Config::find().paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

/// List config entries with pagination, optional category filter and name search
#[tracing::instrument(skip_all)]
pub async fn find_filtered(
    db: &DatabaseConnection,
    params: &ConfigQueryParams,
) -> AppResult<(Vec<config::Model>, u64)> {
    let mut query = Config::find();

    if let Some(ref category) = params.category {
        query = query.filter(Column::Category.eq(category));
    }
    if let Some(ref search) = params.search {
        query = query.filter(Column::Name.contains(search));
    }

    let paginator = query.paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

#[derive(Debug, FromQueryResult)]
struct CategoryCount {
    category: String,
    count: i64,
}

/// List distinct categories with their config entry counts
#[tracing::instrument(skip_all)]
pub async fn list_categories(db: &DatabaseConnection) -> AppResult<Vec<CategoryCountResponse>> {
    use sea_orm::sea_query::Expr;

    let rows = Config::find()
        .select_only()
        .column(Column::Category)
        .column_as(Expr::col(Column::Id).count(), "count")
        .group_by(Column::Category)
        .order_by_asc(Column::Category)
        .into_model::<CategoryCount>()
        .all(db)
        .await?;

    Ok(rows
        .into_iter()
        .map(|r| CategoryCountResponse {
            category: r.category,
            count: r.count as u64,
        })
        .collect())
}

/// Find a config entry by name
#[tracing::instrument(skip_all)]
pub async fn find_by_name(db: &DatabaseConnection, name: &str) -> AppResult<Option<config::Model>> {
    let item = Config::find().filter(Column::Name.eq(name)).one(db).await?;
    Ok(item)
}

/// Update a config value by name
#[tracing::instrument(skip_all)]
pub async fn update_value(
    db: &DatabaseConnection,
    name: &str,
    value: &str,
) -> AppResult<Option<config::Model>> {
    use sea_orm::{ActiveModelTrait, Set};
    if let Some(model) = find_by_name(db, name).await? {
        let mut active: config::ActiveModel = model.into();
        active.value = Set(value.to_string());
        let updated = active.update(db).await?;
        Ok(Some(updated))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
    use std::collections::BTreeMap;

    fn mk_config(name: &str, value: &str) -> config::Model {
        config::Model {
            id: 1,
            name: name.to_string(),
            value: value.to_string(),
            r#type: "string".to_string(),
            default_value: None,
            hint: None,
            pattern: None,
            format: None,
            prompt: None,
            help: None,
            category: "General".to_string(),
            readonly: 0,
            private: 0,
            system: 0,
            requires: None,
        }
    }

    #[tokio::test]
    async fn test_get_config_value_found() {
        let key = ZM_VERSION_KEY;
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<config::Model, _, _>(vec![vec![mk_config(key, "1.36.33")]])
            .into_connection();

        let v = get_config_value(&db, key).await.unwrap();
        assert_eq!(v.as_deref(), Some("1.36.33"));
    }

    #[tokio::test]
    async fn test_get_config_value_not_found() {
        let key = ZM_VERSION_KEY;
        let empty: Vec<config::Model> = vec![];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<config::Model, _, _>(vec![empty])
            .into_connection();

        let v = get_config_value(&db, key).await.unwrap();
        assert!(v.is_none());
    }

    #[tokio::test]
    async fn test_get_zm_version_default_unknown() {
        let empty: Vec<config::Model> = vec![];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<config::Model, _, _>(vec![empty])
            .into_connection();

        let v = get_zm_version(&db).await.unwrap();
        assert_eq!(v, "Unknown");
    }

    #[tokio::test]
    async fn test_update_value_updates_existing() {
        let key = "SomeKey";
        // First query returns existing row; then an exec result for UPDATE
        let db = MockDatabase::new(DatabaseBackend::MySql)
            // initial SELECT find_by_name
            .append_query_results::<config::Model, _, _>(vec![vec![mk_config(key, "old")]])
            // UPDATE exec
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            // some drivers perform a follow-up SELECT; provide updated row
            .append_query_results::<config::Model, _, _>(vec![vec![mk_config(key, "new")]])
            .into_connection();

        let updated = update_value(&db, key, "new").await.unwrap();
        assert!(updated.is_some());
        assert_eq!(updated.unwrap().value, "new");
    }

    #[tokio::test]
    async fn test_update_value_missing_returns_none() {
        let empty: Vec<config::Model> = vec![];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<config::Model, _, _>(vec![empty])
            .into_connection();

        let updated = update_value(&db, "Missing", "x").await.unwrap();
        assert!(updated.is_none());
    }

    fn mk_config_cat(id: u16, name: &str, category: &str) -> config::Model {
        config::Model {
            id,
            name: name.to_string(),
            value: "v".to_string(),
            r#type: "string".to_string(),
            default_value: None,
            hint: None,
            pattern: None,
            format: None,
            prompt: None,
            help: None,
            category: category.to_string(),
            readonly: 0,
            private: 0,
            system: 0,
            requires: None,
        }
    }

    #[tokio::test]
    async fn test_find_filtered_no_filters() {
        let items = vec![
            mk_config_cat(1, "A", "System"),
            mk_config_cat(2, "B", "Network"),
        ];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![vec![BTreeMap::from([(
                "num_items".to_string(),
                sea_orm::Value::Int(Some(2)),
            )])]])
            .append_query_results::<config::Model, _, _>(vec![items])
            .into_connection();

        let params = ConfigQueryParams::default();
        let (rows, total) = find_filtered(&db, &params).await.unwrap();
        assert_eq!(total, 2);
        assert_eq!(rows.len(), 2);
    }

    #[tokio::test]
    async fn test_find_filtered_with_category() {
        let items = vec![mk_config_cat(1, "ZM_OPT_X", "System")];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![vec![BTreeMap::from([(
                "num_items".to_string(),
                sea_orm::Value::Int(Some(1)),
            )])]])
            .append_query_results::<config::Model, _, _>(vec![items])
            .into_connection();

        let params = ConfigQueryParams {
            category: Some("System".into()),
            ..Default::default()
        };
        let (rows, total) = find_filtered(&db, &params).await.unwrap();
        assert_eq!(total, 1);
        assert_eq!(rows[0].category, "System");
    }

    #[tokio::test]
    async fn test_find_filtered_with_search() {
        let items = vec![mk_config_cat(1, "ZM_OPT_USE_AUTH", "System")];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![vec![BTreeMap::from([(
                "num_items".to_string(),
                sea_orm::Value::Int(Some(1)),
            )])]])
            .append_query_results::<config::Model, _, _>(vec![items])
            .into_connection();

        let params = ConfigQueryParams {
            search: Some("AUTH".into()),
            ..Default::default()
        };
        let (rows, total) = find_filtered(&db, &params).await.unwrap();
        assert_eq!(total, 1);
        assert!(rows[0].name.contains("AUTH"));
    }

    #[tokio::test]
    async fn test_list_categories() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![vec![
                BTreeMap::from([
                    (
                        "category".to_string(),
                        sea_orm::Value::String(Some(Box::new("Network".to_string()))),
                    ),
                    ("count".to_string(), sea_orm::Value::BigInt(Some(10))),
                ]),
                BTreeMap::from([
                    (
                        "category".to_string(),
                        sea_orm::Value::String(Some(Box::new("System".to_string()))),
                    ),
                    ("count".to_string(), sea_orm::Value::BigInt(Some(25))),
                ]),
            ]])
            .into_connection();

        let cats = list_categories(&db).await.unwrap();
        assert_eq!(cats.len(), 2);
        assert_eq!(cats[0].category, "Network");
        assert_eq!(cats[0].count, 10);
        assert_eq!(cats[1].category, "System");
        assert_eq!(cats[1].count, 25);
    }
}
