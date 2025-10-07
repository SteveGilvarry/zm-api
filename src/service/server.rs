use tracing::{info, error};
use crate::dto::response::VersionResponse;
use crate::error::{AppResult, AppError};
use crate::server::state::AppState;
use crate::repo;
use crate::constant::API_VERSION;
use tokio::process::Command;

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
        db_version,
    };
    
    Ok(response)
}

/// Restart ZoneMinder daemon
pub async fn restart_zoneminder(_state: &AppState) -> AppResult<()> {
    info!("Restarting ZoneMinder daemon");
    
    // Try systemctl first (most common on modern Linux systems)
    let result = Command::new("systemctl")
        .arg("restart")
        .arg("zoneminder")
        .output()
        .await;
    
    match result {
        Ok(output) if output.status.success() => {
            info!("Successfully restarted ZoneMinder via systemctl");
            Ok(())
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to restart ZoneMinder: {}", stderr);
            
            // Try zmcontrol.pl as fallback
            restart_via_zmcontrol().await
        }
        Err(e) => {
            error!("Failed to execute systemctl: {}", e);
            // Try zmcontrol.pl as fallback
            restart_via_zmcontrol().await
        }
    }
}

/// Stop ZoneMinder daemon
pub async fn stop_zoneminder(_state: &AppState) -> AppResult<()> {
    info!("Stopping ZoneMinder daemon");
    
    let result = Command::new("systemctl")
        .arg("stop")
        .arg("zoneminder")
        .output()
        .await;
    
    match result {
        Ok(output) if output.status.success() => {
            info!("Successfully stopped ZoneMinder via systemctl");
            Ok(())
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to stop ZoneMinder: {}", stderr);
            stop_via_zmcontrol().await
        }
        Err(e) => {
            error!("Failed to execute systemctl: {}", e);
            stop_via_zmcontrol().await
        }
    }
}

/// Start ZoneMinder daemon
pub async fn start_zoneminder(_state: &AppState) -> AppResult<()> {
    info!("Starting ZoneMinder daemon");
    
    let result = Command::new("systemctl")
        .arg("start")
        .arg("zoneminder")
        .output()
        .await;
    
    match result {
        Ok(output) if output.status.success() => {
            info!("Successfully started ZoneMinder via systemctl");
            Ok(())
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to start ZoneMinder: {}", stderr);
            start_via_zmcontrol().await
        }
        Err(e) => {
            error!("Failed to execute systemctl: {}", e);
            start_via_zmcontrol().await
        }
    }
}

/// Fallback: restart via zmcontrol.pl
async fn restart_via_zmcontrol() -> AppResult<()> {
    let output = Command::new("zmcontrol.pl")
        .arg("restart")
        .output()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to execute zmcontrol.pl: {}", e)))?;
    
    if output.status.success() {
        info!("Successfully restarted ZoneMinder via zmcontrol.pl");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::InternalServerError(format!("Failed to restart ZoneMinder: {}", stderr)))
    }
}

/// Fallback: stop via zmcontrol.pl
async fn stop_via_zmcontrol() -> AppResult<()> {
    let output = Command::new("zmcontrol.pl")
        .arg("stop")
        .output()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to execute zmcontrol.pl: {}", e)))?;
    
    if output.status.success() {
        info!("Successfully stopped ZoneMinder via zmcontrol.pl");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::InternalServerError(format!("Failed to stop ZoneMinder: {}", stderr)))
    }
}

/// Fallback: start via zmcontrol.pl
async fn start_via_zmcontrol() -> AppResult<()> {
    let output = Command::new("zmcontrol.pl")
        .arg("start")
        .output()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to execute zmcontrol.pl: {}", e)))?;
    
    if output.status.success() {
        info!("Successfully started ZoneMinder via zmcontrol.pl");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::InternalServerError(format!("Failed to start ZoneMinder: {}", stderr)))
    }
}

// TODO: Implement additional server-related functions here

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase};
    use crate::entity::config::Model as ConfigModel;
    use crate::repo::config::{ZM_VERSION_KEY, ZM_DB_VERSION_KEY};

    fn mk(name: &str, val: &str) -> ConfigModel {
        ConfigModel {
            id: 1,
            name: name.into(),
            value: val.into(),
            r#type: "string".into(),
            default_value: None,
            hint: None,
            pattern: None,
            format: None,
            prompt: None,
            help: None,
            category: "General".into(),
            readonly: 0,
            private: 0,
            system: 0,
            requires: None,
        }
    }

    #[tokio::test]
    async fn test_get_version_ok() {
        // Two queries: one for version and one for db_version
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<ConfigModel, _, _>(vec![vec![mk(ZM_VERSION_KEY, "1.2.3")]])
            .append_query_results::<ConfigModel, _, _>(vec![vec![mk(ZM_DB_VERSION_KEY, "1.2.0")]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let out = get_version(&state).await.unwrap();
        assert_eq!(out.version, "1.2.3");
        assert_eq!(out.db_version, "1.2.0");
        assert!(!out.api_version.is_empty());
    }
}
