use crate::dto::request::events_tags::CreateEventTagRequest;
use crate::dto::response::EventTagResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;
use crate::service::monitor_acl::MonitorScope;
use crate::util::authz::Level;

fn event_tag_not_found(tag_id: u64, event_id: u64) -> AppError {
    AppError::NotFoundError(Resource {
        details: vec![
            ("tag_id".into(), tag_id.to_string()),
            ("event_id".into(), event_id.to_string()),
        ],
        resource_type: ResourceType::EventTag,
    })
}

/// Resolve an event's monitor and confirm the caller may access it at
/// `required`. Returns the same not-found error as a truly missing event tag
/// so a restricted caller cannot tell a hidden monitor's events apart from
/// non-existent ones.
async fn ensure_event_visible(
    state: &AppState,
    scope: &MonitorScope,
    tag_id: u64,
    event_id: u64,
    required: Level,
) -> AppResult<()> {
    // `MonitorScope::All` short-circuits — no lookup needed for the common
    // unrestricted case.
    if !scope.is_restricted() {
        return Ok(());
    }
    let event = repo::events::find_by_id(state, event_id)
        .await?
        .ok_or_else(|| event_tag_not_found(tag_id, event_id))?;
    if !scope.allows(event.monitor_id, required) {
        return Err(event_tag_not_found(tag_id, event_id));
    }
    Ok(())
}

pub async fn list_all(
    state: &AppState,
    scope: &MonitorScope,
    event_id: Option<u64>,
    tag_id: Option<u64>,
) -> AppResult<Vec<EventTagResponse>> {
    let items = repo::events_tags::find_all(state.db(), event_id, tag_id).await?;
    let items = filter_visible(state, scope, items).await?;
    Ok(items.iter().map(EventTagResponse::from).collect())
}

pub async fn list_paginated(
    state: &AppState,
    scope: &MonitorScope,
    event_id: Option<u64>,
    tag_id: Option<u64>,
    params: &PaginationParams,
) -> AppResult<PaginatedResponse<EventTagResponse>> {
    // Row-level ACL is applied per event association, which the composite
    // (event_id, tag_id) table cannot express as a single SQL filter. For an
    // unrestricted caller this is the plain paginated query; a restricted
    // caller's associations are filtered to visible monitors after fetch.
    if !scope.is_restricted() {
        let (items, total) =
            repo::events_tags::find_paginated(state.db(), event_id, tag_id, params).await?;
        let responses: Vec<EventTagResponse> = items.iter().map(EventTagResponse::from).collect();
        return Ok(PaginatedResponse::from_params(responses, total, params));
    }

    let all = repo::events_tags::find_all(state.db(), event_id, tag_id).await?;
    let visible = filter_visible(state, scope, all).await?;
    let total = visible.len() as u64;
    let start = ((params.page().saturating_sub(1)) * params.page_size()) as usize;
    let page: Vec<EventTagResponse> = visible
        .iter()
        .skip(start)
        .take(params.page_size() as usize)
        .map(EventTagResponse::from)
        .collect();
    Ok(PaginatedResponse::from_params(page, total, params))
}

/// Drop associations whose event belongs to a monitor the caller cannot view.
/// Resolves each distinct event's monitor once.
async fn filter_visible(
    state: &AppState,
    scope: &MonitorScope,
    items: Vec<crate::entity::events_tags::Model>,
) -> AppResult<Vec<crate::entity::events_tags::Model>> {
    use std::collections::HashMap;
    if !scope.is_restricted() {
        return Ok(items);
    }
    let mut visible_by_event: HashMap<u64, bool> = HashMap::new();
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let allowed = match visible_by_event.get(&item.event_id) {
            Some(v) => *v,
            None => {
                let allowed = match repo::events::find_by_id(state, item.event_id).await? {
                    Some(ev) => scope.allows(ev.monitor_id, Level::View),
                    None => false,
                };
                visible_by_event.insert(item.event_id, allowed);
                allowed
            }
        };
        if allowed {
            out.push(item);
        }
    }
    Ok(out)
}

pub async fn get_by_id(
    state: &AppState,
    scope: &MonitorScope,
    tag_id: u64,
    event_id: u64,
) -> AppResult<EventTagResponse> {
    ensure_event_visible(state, scope, tag_id, event_id, Level::View).await?;
    let item = repo::events_tags::find_by_composite_id(state.db(), tag_id, event_id).await?;
    let item = item.ok_or_else(|| event_tag_not_found(tag_id, event_id))?;
    Ok(EventTagResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    scope: &MonitorScope,
    req: CreateEventTagRequest,
) -> AppResult<EventTagResponse> {
    // Tagging an event is a write against that event's monitor.
    ensure_event_visible(state, scope, req.tag_id, req.event_id, Level::Edit).await?;

    // Check if association already exists
    let existing =
        repo::events_tags::find_by_composite_id(state.db(), req.tag_id, req.event_id).await?;
    if existing.is_some() {
        return Err(AppError::ResourceExistsError(Resource {
            details: vec![
                ("tag_id".into(), req.tag_id.to_string()),
                ("event_id".into(), req.event_id.to_string()),
            ],
            resource_type: ResourceType::EventTag,
        }));
    }

    let model = repo::events_tags::create(state.db(), &req).await?;
    Ok(EventTagResponse::from(&model))
}

pub async fn delete(
    state: &AppState,
    scope: &MonitorScope,
    tag_id: u64,
    event_id: u64,
) -> AppResult<()> {
    ensure_event_visible(state, scope, tag_id, event_id, Level::Edit).await?;
    let ok = repo::events_tags::delete_by_composite_id(state.db(), tag_id, event_id).await?;
    if ok {
        Ok(())
    } else {
        Err(event_tag_not_found(tag_id, event_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::events::Model as EventModel;
    use sea_orm::{DatabaseBackend, MockDatabase};
    use std::collections::HashMap;

    fn mk_event(id: u64, monitor_id: u32) -> EventModel {
        use crate::entity::sea_orm_active_enums::{Orientation, Scheme};
        use rust_decimal::Decimal;
        EventModel {
            id,
            monitor_id,
            storage_id: Some(1),
            secondary_storage_id: None,
            name: "ev".into(),
            cause: None,
            start_date_time: None,
            end_date_time: None,
            width: 0,
            height: 0,
            length: Decimal::new(0, 0),
            frames: Some(0),
            alarm_frames: Some(0),
            default_video: "v.mp4".into(),
            save_jpe_gs: None,
            tot_score: 0,
            avg_score: None,
            max_score: None,
            max_score_frame_id: None,
            archived: 0,
            videoed: 0,
            uploaded: 0,
            emailed: 0,
            messaged: 0,
            executed: 0,
            notes: None,
            state_id: 1,
            orientation: Orientation::Rotate0,
            disk_space: None,
            scheme: Scheme::Deep,
            locked: 0,
            latitude: None,
            longitude: None,
        }
    }

    /// A restricted caller resolves the event's monitor; when that monitor is
    /// outside their scope the tag lookup is a not-found (existence hidden),
    /// and the association query never runs.
    #[tokio::test]
    async fn restricted_scope_hides_tag_of_out_of_scope_event() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<EventModel, _, _>(vec![vec![mk_event(42, 7)]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        // Caller may see monitor 1 only; the event belongs to monitor 7.
        let mut map = HashMap::new();
        map.insert(1u32, Level::View);
        let scope = MonitorScope::Restricted(map);

        let err = get_by_id(&state, &scope, 5, 42)
            .await
            .expect_err("out-of-scope event tag must be hidden");
        assert!(matches!(err, AppError::NotFoundError(_)), "got {err:?}");
    }

    /// A restricted caller whose scope *does* include the event's monitor gets
    /// through the ACL check to the association lookup (which then 404s here
    /// because the mock returns no association row).
    #[tokio::test]
    async fn restricted_scope_allows_in_scope_event() {
        use crate::entity::events_tags::Model as EventTagModel;
        let empty: Vec<EventTagModel> = vec![];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<EventModel, _, _>(vec![vec![mk_event(42, 1)]])
            .append_query_results::<EventTagModel, _, _>(vec![empty])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let mut map = HashMap::new();
        map.insert(1u32, Level::View);
        let scope = MonitorScope::Restricted(map);

        // Passes the ACL gate; the missing association is a normal not-found.
        let err = get_by_id(&state, &scope, 5, 42)
            .await
            .expect_err("no association row -> not found");
        assert!(matches!(err, AppError::NotFoundError(_)), "got {err:?}");
    }
}
