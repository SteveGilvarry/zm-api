use crate::dto::request::events::SortDirection;
use crate::dto::request::filter_ast::FilterQuery;
use crate::dto::response::events::{EventResponse, PaginatedEventsResponse};
use crate::dto::response::events_tags::TagSummary;
use crate::dto::response::FilterResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::entity::{events, filters};
use crate::error::{AppError, AppResult};
use crate::repo;
use crate::repo::events as events_repo;
use crate::server::state::AppState;
use crate::service::monitor_acl::MonitorScope;
use crate::util::authz::Level;

/// Build a `FilterResponse`, additionally parsing the stored `query_json` into
/// the structured AST for display. A parse failure (legacy/unmodelled filter)
/// just leaves `filter` as `None` rather than failing the request.
fn response_with_ast(model: &filters::Model) -> FilterResponse {
    let mut resp = FilterResponse::from(model);
    resp.filter = crate::service::filter_translate::from_zm_query_json(&model.query_json)
        .ok()
        .flatten();
    resp
}

pub async fn list_all(state: &AppState) -> AppResult<Vec<FilterResponse>> {
    let items = repo::filters::find_all(state.db()).await?;
    Ok(items.iter().map(response_with_ast).collect())
}

pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
) -> AppResult<PaginatedResponse<FilterResponse>> {
    let (items, total) = repo::filters::find_paginated(state.db(), params).await?;
    let responses: Vec<FilterResponse> = items.iter().map(response_with_ast).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<FilterResponse> {
    let item = repo::filters::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(crate::error::Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: crate::error::ResourceType::Message,
        })
    })?;
    Ok(response_with_ast(&item))
}

/// Run a structured filter immediately and return the matching events
/// (paginated). The predicate is compiled to a parameterised query — no SQL is
/// built from client strings — and row-level monitor ACL is applied.
pub async fn preview(
    state: &AppState,
    query: &FilterQuery,
    params: &PaginationParams,
    scope: &MonitorScope,
) -> AppResult<PaginatedEventsResponse> {
    crate::service::filter_translate::validate(query)?;
    let condition = crate::service::filter_build::build_condition(&query.predicate)?;
    let sort_column = match &query.sort {
        Some(s) => crate::service::filter_build::event_sort_column(s.field)?,
        None => events::Column::StartDateTime,
    };
    let sort_direction = query
        .sort
        .as_ref()
        .map(|s| s.dir)
        .unwrap_or(SortDirection::Desc);

    let page = params.page();
    let page_size = params.page_size();
    let monitor_filter = scope.visible_ids(Level::View);

    let (event_models, total) = events_repo::find_with_condition(
        state,
        condition,
        sort_column,
        sort_direction,
        monitor_filter,
        page - 1,
        page_size,
    )
    .await?;

    // Attach tags, mirroring `service::events::list`.
    let event_ids: Vec<u64> = event_models.iter().map(|e| e.id).collect();
    let tags_map = events_repo::find_tags_for_events(state, &event_ids).await?;
    let items = event_models
        .into_iter()
        .map(|e| {
            let tags: Vec<TagSummary> = tags_map
                .get(&e.id)
                .map(|t| t.iter().map(TagSummary::from).collect())
                .unwrap_or_default();
            EventResponse::with_tags(e, tags)
        })
        .collect();

    Ok(PaginatedEventsResponse {
        items,
        total,
        per_page: page_size,
        current_page: page,
        last_page: total.div_ceil(page_size),
    })
}

pub async fn update(
    state: &AppState,
    id: u32,
    req: &crate::dto::request::UpdateFilterRequest,
) -> AppResult<FilterResponse> {
    // A structured AST, when supplied, is translated to ZoneMinder's flat
    // query_json (and validated by construction during translation).
    let mut effective = req.clone();
    if let Some(ast) = &req.filter {
        effective.query_json = Some(crate::service::filter_translate::to_zm_query_json(ast)?);
    }
    // Guard against stored SQL injection on any raw query_json: it is turned
    // into live SQL by zmfilter.pl. See `service::filter_query`.
    if let Some(q) = effective.query_json.as_deref() {
        crate::service::filter_query::validate_query_json(q)?;
    }
    let item = repo::filters::update(state.db(), id, &effective).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(crate::error::Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: crate::error::ResourceType::Message,
        })
    })?;
    Ok(response_with_ast(&item))
}

pub async fn create(
    state: &AppState,
    req: crate::dto::request::CreateFilterRequest,
) -> AppResult<FilterResponse> {
    // A structured AST, when supplied, is translated to ZoneMinder's flat
    // query_json and wins over any raw string.
    let mut req = req;
    if let Some(ast) = &req.filter {
        req.query_json = crate::service::filter_translate::to_zm_query_json(ast)?;
    }
    // Guard against stored SQL injection: query_json is turned into live SQL by
    // zmfilter.pl. See `service::filter_query`.
    crate::service::filter_query::validate_query_json(&req.query_json)?;
    let model = repo::filters::create(state.db(), &req).await?;
    Ok(response_with_ast(&model))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::filters::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(crate::error::AppError::NotFoundError(
            crate::error::Resource {
                details: vec![("id".into(), id.to_string())],
                resource_type: crate::error::ResourceType::Message,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::filters::Model as FilterModel;
    use crate::error::AppError;
    use crate::server::state::AppState;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    fn mk_filter(id: u32, name: &str) -> FilterModel {
        use crate::entity::sea_orm_active_enums::EmailFormat;
        FilterModel {
            id,
            name: name.into(),
            user_id: Some(1),
            execute_interval: 0,
            query_json: "{}".into(),
            auto_archive: 0,
            auto_unarchive: 0,
            auto_video: 0,
            auto_upload: 0,
            auto_email: 0,
            email_server: None,
            email_to: None,
            email_subject: None,
            email_body: None,
            email_format: EmailFormat::Summary,
            auto_message: 0,
            auto_execute: 0,
            auto_execute_cmd: None,
            auto_delete: 0,
            auto_move: 0,
            auto_copy: 0,
            auto_copy_to: 0,
            auto_move_to: 0,
            update_disk_space: 0,
            background: 0,
            concurrent: 0,
            lock_rows: 0,
        }
    }

    #[tokio::test]
    async fn test_get_by_id_ok() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<FilterModel, _, _>(vec![vec![mk_filter(1, "f1")]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let resp = get_by_id(&state, 1).await.unwrap();
        assert_eq!(resp.id, 1);
        assert_eq!(resp.name, "f1");
    }

    #[tokio::test]
    async fn test_update_ok() {
        let initial = mk_filter(2, "old");
        let updated = FilterModel {
            name: "new".into(),
            query_json: "{\"k\":\"v\"}".into(),
            ..initial.clone()
        };
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<FilterModel, _, _>(vec![vec![initial]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_query_results::<FilterModel, _, _>(vec![vec![updated.clone()]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let req = crate::dto::request::UpdateFilterRequest {
            name: Some("new".into()),
            query_json: Some("{\"k\":\"v\"}".into()),
            ..Default::default()
        };
        let resp = update(&state, 2, &req).await.unwrap();
        assert_eq!(resp.name, "new");
        assert_eq!(resp.query_json, "{\"k\":\"v\"}");
    }

    #[tokio::test]
    async fn test_delete_ok() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        assert!(delete(&state, 1).await.is_ok());
    }

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let empty: Vec<FilterModel> = vec![];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<FilterModel, _, _>(vec![empty])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let err = get_by_id(&state, 99).await.expect_err("should error");
        matches!(err, AppError::NotFoundError(_));
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let empty: Vec<FilterModel> = vec![];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<FilterModel, _, _>(vec![empty])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let req = crate::dto::request::UpdateFilterRequest {
            name: Some("x".into()),
            ..Default::default()
        };
        let err = update(&state, 1, &req).await.expect_err("should error");
        matches!(err, AppError::NotFoundError(_));
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 0,
            }])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let err = delete(&state, 1).await.expect_err("should error");
        matches!(err, AppError::NotFoundError(_));
    }
}
