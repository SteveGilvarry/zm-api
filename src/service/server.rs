use tracing::info;
use crate::dto::response::VersionResponse;
use crate::error::AppResult;
use crate::server::state::AppState;
use crate::repo;
use crate::constant::API_VERSION;

// API version (from Cargo.toml via constant)

pub async fn get_version(state: &AppState) -> AppResult<VersionResponse> {
    info!("Getting ZoneMinder and API version information");
    
    // Fetch version information from the database
    let zm_version = repo::config::get_zm_version(state.db()).await?;
    let db_version = repo::config::get_zm_db_version(state.db()).await?;
    
    // Build version response
    let response = VersionResponse {
        version: zm_version,
        api_version: API_VERSION.to_string(),
        db_version: db_version,
    };
    
    Ok(response)
}

// TODO: Implement additional server-related functions here