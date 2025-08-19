use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use crate::entity::config::{Entity as Config, Column};
use crate::error::AppResult;

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