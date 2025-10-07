use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use crate::entity::config::{Entity as Config, Column};
use crate::error::AppResult;
use crate::entity::config;

// Constants for config keys
pub const ZM_VERSION_KEY: &str = "ZM_DYN_CURR_VERSION";
pub const ZM_DB_VERSION_KEY: &str = "ZM_DYN_DB_VERSION";

/// Fetch a specific config value by name
#[tracing::instrument(skip_all)]
pub async fn get_config_value(
    db: &DatabaseConnection,
    name: &str,
) -> AppResult<Option<String>> {
    let config = Config::find()
        .filter(Column::Name.eq(name))
        .one(db)
        .await?;
    
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

/// Find a config entry by name
#[tracing::instrument(skip_all)]
pub async fn find_by_name(db: &DatabaseConnection, name: &str) -> AppResult<Option<config::Model>> {
    let item = Config::find().filter(Column::Name.eq(name)).one(db).await?;
    Ok(item)
}

/// Update a config value by name
#[tracing::instrument(skip_all)]
pub async fn update_value(db: &DatabaseConnection, name: &str, value: &str) -> AppResult<Option<config::Model>> {
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
            .append_exec_results(vec![MockExecResult { last_insert_id: 0, rows_affected: 1 }])
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
}
