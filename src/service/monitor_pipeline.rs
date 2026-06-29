//! Service for a monitor's stored zm-next processing graph (the "free graph").
//!
//! The graph is the part of the pipeline ZoneMinder has no schema for; it is
//! validated here ([`crate::service::zmnext::graph`]), persisted to the
//! `monitor_pipeline` table, and composed with the monitor-derived capture/store
//! nodes at worker spawn (`service::zmnext::pipeline::compose_pipeline`). A
//! successful write best-effort restarts the worker so the change takes effect.

use serde_json::Value;

use crate::dto::response::monitor_pipeline::MonitorPipelineResponse;
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;
use crate::service::monitor_acl::MonitorScope;
use crate::service::zmnext::graph;

/// Fetch a monitor's stored processing graph, after verifying the caller can
/// access the monitor. 404 if the monitor is unknown/forbidden or has no graph.
pub async fn get(
    state: &AppState,
    monitor_id: u32,
    scope: &MonitorScope,
) -> AppResult<MonitorPipelineResponse> {
    // Enforce monitor row ACL + existence (404 for unknown/forbidden monitor).
    crate::service::monitor::get_by_id(state, monitor_id, scope).await?;
    repo::monitor_pipeline::find_by_monitor(state.db(), monitor_id)
        .await?
        .map(MonitorPipelineResponse::from)
        .ok_or_else(|| {
            AppError::NotFoundError(Resource {
                details: vec![("monitor_id".into(), monitor_id.to_string())],
                resource_type: ResourceType::Monitor,
            })
        })
}

/// Validate and replace a monitor's processing graph, then best-effort restart
/// its worker. The graph document is `{ "plugins": [...] }`. Verifies the monitor
/// exists/accessible (404) before writing, so we never persist an orphan row.
pub async fn replace(
    state: &AppState,
    monitor_id: u32,
    graph_doc: Value,
    scope: &MonitorScope,
) -> AppResult<MonitorPipelineResponse> {
    crate::service::monitor::get_by_id(state, monitor_id, scope).await?;
    graph::validate_graph(&graph_doc).map_err(AppError::BadRequestError)?;
    let body = serde_json::to_string(&graph_doc)?;
    let now = chrono::Utc::now().naive_utc();
    let row = repo::monitor_pipeline::upsert(state.db(), monitor_id, body, 1, now).await?;
    reload_worker(state, monitor_id).await;
    Ok(MonitorPipelineResponse::from(row))
}

/// Remove a monitor's stored graph (reverts it to the default generated
/// pipeline), then best-effort restart its worker.
pub async fn delete(state: &AppState, monitor_id: u32, scope: &MonitorScope) -> AppResult<()> {
    crate::service::monitor::get_by_id(state, monitor_id, scope).await?;
    repo::monitor_pipeline::delete_by_monitor(state.db(), monitor_id).await?;
    reload_worker(state, monitor_id).await;
    Ok(())
}

/// "Make this monitor zm-next": set `UseZmNext=1` and, if it has no stored graph
/// yet, materialize the default processing graph so it behaves like the legacy
/// default until edited. Best-effort restarts the worker. Reversible via
/// [`disable_zmnext`]. Errors if the `UseZmNext` column is absent (the ZoneMinder
/// fork migration is required).
pub async fn enable_zmnext(
    state: &AppState,
    monitor_id: u32,
    scope: &MonitorScope,
) -> AppResult<MonitorPipelineResponse> {
    crate::service::monitor::get_by_id(state, monitor_id, scope).await?;
    crate::repo::monitors::set_use_zmnext(state.db(), monitor_id, true)
        .await
        .map_err(|e| {
            AppError::BadRequestError(format!(
                "could not set UseZmNext for monitor {monitor_id} \
                 (the ZoneMinder fork migration adding this column may be required): {e}"
            ))
        })?;

    // Seed a default graph only if one doesn't already exist (re-enabling keeps
    // any previously configured graph).
    let row = match repo::monitor_pipeline::find_by_monitor(state.db(), monitor_id).await? {
        Some(existing) => existing,
        None => {
            let synopsis = state.config.synopsis.enabled
                && state.config.synopsis.enabled_monitors.contains(&monitor_id);
            let graph = crate::service::zmnext::pipeline::default_processing_graph(
                &state.config.zmnext.pipeline,
                synopsis,
            );
            let body = serde_json::to_string(&graph)?;
            let now = chrono::Utc::now().naive_utc();
            repo::monitor_pipeline::upsert(state.db(), monitor_id, body, 1, now).await?
        }
    };
    reload_worker(state, monitor_id).await;
    Ok(MonitorPipelineResponse::from(row))
}

/// Revert a monitor from zm-next: clear `UseZmNext` so legacy zmc/zma resume. The
/// stored graph is kept (dormant) so re-enabling restores the configuration.
pub async fn disable_zmnext(
    state: &AppState,
    monitor_id: u32,
    scope: &MonitorScope,
) -> AppResult<()> {
    crate::service::monitor::get_by_id(state, monitor_id, scope).await?;
    crate::repo::monitors::set_use_zmnext(state.db(), monitor_id, false)
        .await
        .map_err(|e| {
            AppError::BadRequestError(format!(
                "could not clear UseZmNext for monitor {monitor_id}: {e}"
            ))
        })?;
    reload_worker(state, monitor_id).await;
    Ok(())
}

/// Best-effort: restart the monitor's worker so a graph change is applied now.
/// Never fails the request — the change is already persisted and applies on the
/// next (re)start regardless.
async fn reload_worker(state: &AppState, monitor_id: u32) {
    if let Some(mgr) = &state.daemon_manager {
        if let Err(e) = mgr.restart_monitor(monitor_id).await {
            tracing::warn!(
                "monitor {monitor_id} pipeline graph saved; worker restart failed \
                 (applies on next start): {e}"
            );
        }
    }
}
