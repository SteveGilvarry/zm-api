use crate::dto::response::GroupResponse;
use crate::dto::{PaginatedResponse, PaginationParams};
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;
use crate::service::group_acl::GroupScope;
use crate::util::authz::Level;

/// The 404 returned both for a genuinely missing group and for a group the
/// caller's [`GroupScope`] hides — identical, so scope never leaks existence.
fn not_found(id: u32) -> AppError {
    AppError::NotFoundError(Resource {
        details: vec![("id".into(), id.to_string())],
        resource_type: ResourceType::User,
    })
}

pub async fn list_all(state: &AppState) -> AppResult<Vec<GroupResponse>> {
    let items = repo::groups::find_all(state.db()).await?;
    Ok(items.iter().map(GroupResponse::from).collect())
}

pub async fn list_paginated(
    state: &AppState,
    params: &PaginationParams,
    scope: &GroupScope,
) -> AppResult<PaginatedResponse<GroupResponse>> {
    let filter = scope.visible_ids(Level::View);
    let (items, total) =
        repo::groups::find_paginated(state.db(), params, filter.as_deref()).await?;
    let responses: Vec<GroupResponse> = items.iter().map(GroupResponse::from).collect();
    Ok(PaginatedResponse::from_params(responses, total, params))
}

pub async fn get_by_id(state: &AppState, id: u32, scope: &GroupScope) -> AppResult<GroupResponse> {
    if !scope.allows(id, Level::View) {
        return Err(not_found(id));
    }
    let item = repo::groups::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| not_found(id))?;
    Ok(GroupResponse::from(&item))
}

pub async fn update(
    state: &AppState,
    id: u32,
    name: Option<String>,
    scope: &GroupScope,
) -> AppResult<GroupResponse> {
    if !scope.allows(id, Level::Edit) {
        return Err(not_found(id));
    }
    let item = repo::groups::update(state.db(), id, name).await?;
    let item = item.ok_or_else(|| not_found(id))?;
    Ok(GroupResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    req: crate::dto::request::CreateGroupRequest,
) -> AppResult<GroupResponse> {
    let model = repo::groups::create(state.db(), &req).await?;
    Ok(GroupResponse::from(&model))
}

pub async fn delete(state: &AppState, id: u32, scope: &GroupScope) -> AppResult<()> {
    if !scope.allows(id, Level::Edit) {
        return Err(not_found(id));
    }
    let ok = repo::groups::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(not_found(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::groups::Model as GroupModel;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    fn mk_group(id: u32, name: &str) -> GroupModel {
        GroupModel {
            id,
            name: name.into(),
            parent_id: None,
        }
    }

    #[tokio::test]
    async fn test_list_all_ok() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<GroupModel, _, _>(vec![vec![
                mk_group(1, "g1"),
                mk_group(2, "g2"),
            ]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let out = list_all(&state).await.unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].name, "g1");
    }

    #[tokio::test]
    async fn test_get_by_id_ok_and_not_found() {
        let db_ok = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<GroupModel, _, _>(vec![vec![mk_group(9, "ok")]])
            .into_connection();
        let state_ok = AppState::for_test_with_db(db_ok);
        assert_eq!(
            get_by_id(&state_ok, 9, &GroupScope::All).await.unwrap().id,
            9
        );

        let empty: Vec<GroupModel> = vec![];
        let db_none = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<GroupModel, _, _>(vec![empty])
            .into_connection();
        let state_none = AppState::for_test_with_db(db_none);
        assert!(matches!(
            get_by_id(&state_none, 1, &GroupScope::All)
                .await
                .err()
                .unwrap(),
            AppError::NotFoundError(_)
        ));
    }

    #[tokio::test]
    async fn test_update_ok_and_not_found() {
        let initial = mk_group(3, "old");
        let after = mk_group(3, "new");
        let db_ok = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<GroupModel, _, _>(vec![vec![initial]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_query_results::<GroupModel, _, _>(vec![vec![after.clone()]])
            .into_connection();
        let state_ok = AppState::for_test_with_db(db_ok);
        assert_eq!(
            update(&state_ok, 3, Some("new".into()), &GroupScope::All)
                .await
                .unwrap()
                .name,
            "new"
        );

        let empty: Vec<GroupModel> = vec![];
        let db_none = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<GroupModel, _, _>(vec![empty])
            .into_connection();
        let state_none = AppState::for_test_with_db(db_none);
        assert!(matches!(
            update(&state_none, 3, Some("x".into()), &GroupScope::All)
                .await
                .err()
                .unwrap(),
            AppError::NotFoundError(_)
        ));
    }

    #[tokio::test]
    async fn test_create_and_delete() {
        use crate::dto::request::groups::CreateGroupRequest;
        let db_create = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 42,
                rows_affected: 1,
            }])
            .append_query_results::<GroupModel, _, _>(vec![vec![mk_group(42, "new")]])
            .into_connection();
        let state_create = AppState::for_test_with_db(db_create);
        let req = CreateGroupRequest {
            name: "new".into(),
            parent_id: None,
        };
        assert_eq!(create(&state_create, req).await.unwrap().name, "new");

        let db_del_ok = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .into_connection();
        let state_del_ok = AppState::for_test_with_db(db_del_ok);
        assert!(delete(&state_del_ok, 1, &GroupScope::All).await.is_ok());

        let db_del_none = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 0,
            }])
            .into_connection();
        let state_del_none = AppState::for_test_with_db(db_del_none);
        assert!(matches!(
            delete(&state_del_none, 1, &GroupScope::All)
                .await
                .err()
                .unwrap(),
            AppError::NotFoundError(_)
        ));
    }
}
