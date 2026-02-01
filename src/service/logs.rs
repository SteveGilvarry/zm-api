use crate::dto::request::logs::LogQueryParams;
use crate::dto::response::logs::{LogResponse, PaginatedLogsResponse};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::repo::logs::LogQueryOptions;
use crate::server::state::AppState;

/// Default page size for log listing
const DEFAULT_PAGE_SIZE: u64 = 50;

/// List logs with pagination and filtering
pub async fn list(state: &AppState, params: &LogQueryParams) -> AppResult<PaginatedLogsResponse> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(DEFAULT_PAGE_SIZE);

    let options = LogQueryOptions {
        component: params.component.clone(),
        level: params.level,
        server_id: params.server_id,
    };

    let (logs, total) =
        repo::logs::find_with_options(state.db(), options, page - 1, page_size).await?;

    let total_pages = total.div_ceil(page_size);

    Ok(PaginatedLogsResponse {
        logs: logs.iter().map(LogResponse::from).collect(),
        total,
        per_page: page_size,
        current_page: page,
        last_page: total_pages,
    })
}

/// List recent logs (legacy helper)
pub async fn list_recent(state: &AppState, limit: u64) -> AppResult<Vec<LogResponse>> {
    let items = repo::logs::find_all(state.db(), limit).await?;
    Ok(items.iter().map(LogResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<LogResponse> {
    let item = repo::logs::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::Message,
        })
    })?;
    Ok(LogResponse::from(&item))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::logs::Model as LogModel;
    use rust_decimal::Decimal;
    use sea_orm::{DatabaseBackend, MockDatabase};

    fn mk(id: u32, msg: &str) -> LogModel {
        LogModel {
            id,
            time_key: Decimal::new(0, 0),
            component: "zmdc".into(),
            server_id: None,
            pid: None,
            level: 1,
            code: "A01".into(),
            message: msg.into(),
            file: None,
            line: None,
        }
    }

    #[tokio::test]
    async fn test_list_recent_ok() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<LogModel, _, _>(vec![vec![mk(1, "a"), mk(2, "b")]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let out = list_recent(&state, 2).await.unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[1].message, "b");
    }

    #[tokio::test]
    async fn test_get_by_id_ok_and_not_found() {
        let db_ok = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<LogModel, _, _>(vec![vec![mk(7, "ok")]])
            .into_connection();
        let state_ok = AppState::for_test_with_db(db_ok);
        assert_eq!(get_by_id(&state_ok, 7).await.unwrap().id, 7);

        let empty: Vec<LogModel> = vec![];
        let db_none = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<LogModel, _, _>(vec![empty])
            .into_connection();
        let state_none = AppState::for_test_with_db(db_none);
        assert!(matches!(
            get_by_id(&state_none, 1).await.err().unwrap(),
            AppError::NotFoundError(_)
        ));
    }
}
