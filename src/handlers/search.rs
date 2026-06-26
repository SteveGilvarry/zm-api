//! HTTP handlers for natural-language / semantic event search.
//!
//! All retrieval is bounded by the caller's [`MonitorScope`] ACL: a restricted
//! user only ever sees events from monitors they may view, and a request for a
//! monitor outside their scope yields empty results (never a leak). Search is
//! off by default; when disabled the endpoints return `503`.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use garde::Validate;
use tracing::instrument;

use crate::dto::request::search::{SearchQueryParams, SimilarQueryParams};
use crate::dto::response::search::{SearchCitation, SearchResponse, SimilarResponse};
use crate::error::{AppError, AppResponseError, AppResult};
use crate::handlers::events_playback::get_event_entity;
use crate::server::state::AppState;
use crate::service::monitor_acl::MonitorScope;
use crate::service::search::store::Filter;
use crate::service::search::{SearchError, SearchService};
use crate::util::authz::Level;

/// Default page size when the caller omits `k`.
const DEFAULT_K: usize = 10;

/// Map a [`SearchError`] to an HTTP error. `Disabled` is a 503 (the feature is
/// off / no backend); everything else is an internal error.
fn map_search_err(e: SearchError) -> AppError {
    match e {
        SearchError::Disabled => {
            AppError::ServiceUnavailableError("event search is disabled".into())
        }
        other => AppError::InternalServerError(other.to_string()),
    }
}

/// Fetch the search service, or fail with 503 when search is off / no backend.
fn require_search(state: &AppState) -> AppResult<Arc<SearchService>> {
    match state.search_service.as_ref() {
        Some(s) if s.enabled() => Ok(s.clone()),
        _ => Err(AppError::ServiceUnavailableError(
            "event search is disabled".into(),
        )),
    }
}

/// The monitor pre-filter after applying the caller's ACL.
enum ScopeFilter {
    /// Run the query over these monitor ids. An empty vec means "all monitors"
    /// (only reachable for unrestricted callers).
    Run(Vec<u32>),
    /// The caller may see no matching monitors — short-circuit to empty results.
    Empty,
}

/// Intersect the requested monitor filter with the caller's visible monitors.
fn resolve_monitor_ids(scope: &MonitorScope, requested: Option<u32>) -> ScopeFilter {
    match scope.visible_ids(Level::View) {
        // Unrestricted: honour the optional single-monitor filter, else all.
        None => ScopeFilter::Run(requested.into_iter().collect()),
        Some(visible) => match requested {
            // Requested a specific monitor: allow only if it's visible.
            Some(m) if visible.contains(&m) => ScopeFilter::Run(vec![m]),
            Some(_) => ScopeFilter::Empty,
            // No specific monitor: scope to the visible allowlist.
            None if visible.is_empty() => ScopeFilter::Empty,
            None => ScopeFilter::Run(visible),
        },
    }
}

/// Split a comma-separated class filter into normalised, deduped labels.
fn parse_classes(raw: Option<&str>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if let Some(raw) = raw {
        for part in raw.split(',') {
            let c = part.trim().to_lowercase();
            if !c.is_empty() && !out.contains(&c) {
                out.push(c);
            }
        }
    }
    out
}

/// Natural-language event search: hybrid retrieval (vector ANN + lexical FTS,
/// RRF-fused and cross-encoder reranked), an exact count for "how many …"
/// queries, and a grounded NL answer when the router LLM is reachable.
#[utoipa::path(
    get,
    path = "/api/v3/search",
    operation_id = "searchEvents",
    tag = "Search",
    params(
        ("q" = String, Query, description = "Natural-language query", example = "person at the front door at night"),
        ("monitor_id" = Option<u32>, Query, description = "Restrict to one monitor (subject to ACL)", example = 1),
        ("from" = Option<i64>, Query, description = "Inclusive lower time bound, epoch seconds", example = 1700000000_i64),
        ("to" = Option<i64>, Query, description = "Inclusive upper time bound, epoch seconds", example = 1700003600_i64),
        ("class" = Option<String>, Query, description = "Comma-separated object/class pre-filter", example = "person,car"),
        ("k" = Option<usize>, Query, description = "Number of results (1-50, default 10)", example = 10),
    ),
    responses(
        (status = 200, description = "Search results", body = SearchResponse),
        (status = 400, description = "Bad request", body = AppResponseError),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 503, description = "Search disabled / no backend", body = AppResponseError),
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state, scope))]
pub async fn search_events(
    State(state): State<AppState>,
    scope: MonitorScope,
    Query(params): Query<SearchQueryParams>,
) -> AppResult<Json<SearchResponse>> {
    params.validate().map_err(AppError::InvalidInputError)?;
    let search = require_search(&state)?;
    let k = params.k.unwrap_or(DEFAULT_K);
    let backend = search.backend().as_str().to_string();

    let monitor_ids = match resolve_monitor_ids(&scope, params.monitor_id) {
        ScopeFilter::Run(ids) => ids,
        ScopeFilter::Empty => {
            return Ok(Json(SearchResponse {
                answer: None,
                count: None,
                citations: Vec::new(),
                backend,
            }));
        }
    };

    let filter = Filter {
        monitor_ids,
        from: params.from,
        to: params.to,
        classes: parse_classes(params.class.as_deref()),
    };

    let outcome = search
        .answer(&params.q, &filter, k)
        .await
        .map_err(map_search_err)?;
    let citations = outcome
        .citations
        .into_iter()
        .map(SearchCitation::from)
        .collect();
    Ok(Json(SearchResponse {
        answer: outcome.answer,
        count: outcome.count,
        citations,
        backend,
    }))
}

/// "More like this": events nearest the given event's stored embedding. The
/// source event is ACL-checked (a 404 hides events outside the caller's scope),
/// and results are bounded by the same scope.
#[utoipa::path(
    get,
    path = "/api/v3/events/{id}/similar",
    operation_id = "similarEvents",
    tag = "Search",
    params(
        ("id" = u64, Path, description = "Source event id"),
        ("k" = Option<usize>, Query, description = "Number of results (1-50, default 10)", example = 10),
    ),
    responses(
        (status = 200, description = "Similar events", body = SimilarResponse),
        (status = 401, description = "Unauthorized", body = AppResponseError),
        (status = 404, description = "Event not found / not visible", body = AppResponseError),
        (status = 503, description = "Search disabled / no backend", body = AppResponseError),
    ),
    security(("jwt" = []))
)]
#[instrument(skip(state, scope))]
pub async fn similar_events(
    State(state): State<AppState>,
    scope: MonitorScope,
    Path(id): Path<u64>,
    Query(params): Query<SimilarQueryParams>,
) -> AppResult<Json<SimilarResponse>> {
    params.validate().map_err(AppError::InvalidInputError)?;
    let search = require_search(&state)?;
    let k = params.k.unwrap_or(DEFAULT_K);
    let backend = search.backend().as_str().to_string();

    // Row-level ACL + existence: hides events of monitors outside the scope.
    get_event_entity(&state, id, &scope).await?;

    let monitor_ids = match resolve_monitor_ids(&scope, None) {
        ScopeFilter::Run(ids) => ids,
        ScopeFilter::Empty => {
            return Ok(Json(SimilarResponse {
                citations: Vec::new(),
                backend,
            }));
        }
    };
    let filter = Filter {
        monitor_ids,
        ..Default::default()
    };
    let hits = search
        .similar(id, &filter, k)
        .await
        .map_err(map_search_err)?;
    let citations = hits.into_iter().map(SearchCitation::from).collect();
    Ok(Json(SimilarResponse { citations, backend }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::authz::Level;
    use std::collections::HashMap;

    fn restricted(ids: &[(u32, Level)]) -> MonitorScope {
        MonitorScope::Restricted(ids.iter().copied().collect::<HashMap<_, _>>())
    }

    #[test]
    fn parse_classes_normalises_and_dedups() {
        assert_eq!(
            parse_classes(Some("Person, car ,person,")),
            vec!["person".to_string(), "car".to_string()]
        );
        assert!(parse_classes(None).is_empty());
        assert!(parse_classes(Some("  , ,")).is_empty());
    }

    #[test]
    fn unrestricted_scope_passes_request_through() {
        // No filter requested -> all monitors (empty vec = no monitor filter).
        match resolve_monitor_ids(&MonitorScope::All, None) {
            ScopeFilter::Run(ids) => assert!(ids.is_empty()),
            ScopeFilter::Empty => panic!("unrestricted should run"),
        }
        // Specific monitor requested -> that monitor only.
        match resolve_monitor_ids(&MonitorScope::All, Some(7)) {
            ScopeFilter::Run(ids) => assert_eq!(ids, vec![7]),
            ScopeFilter::Empty => panic!("unrestricted should run"),
        }
    }

    #[test]
    fn restricted_scope_enforces_allowlist() {
        let scope = restricted(&[(1, Level::View), (2, Level::View)]);
        // No specific monitor -> scoped to the visible allowlist.
        match resolve_monitor_ids(&scope, None) {
            ScopeFilter::Run(mut ids) => {
                ids.sort();
                assert_eq!(ids, vec![1, 2]);
            }
            ScopeFilter::Empty => panic!("should run over visible ids"),
        }
        // Visible monitor requested -> allowed.
        assert!(matches!(
            resolve_monitor_ids(&scope, Some(1)),
            ScopeFilter::Run(ids) if ids == vec![1]
        ));
        // Non-visible monitor requested -> empty (no leak).
        assert!(matches!(
            resolve_monitor_ids(&scope, Some(99)),
            ScopeFilter::Empty
        ));
    }

    #[test]
    fn restricted_with_no_visible_monitors_is_empty() {
        let scope = restricted(&[]);
        assert!(matches!(
            resolve_monitor_ids(&scope, None),
            ScopeFilter::Empty
        ));
    }
}
