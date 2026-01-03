use crate::dto::response::UserResponse;
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;

pub async fn list_all(state: &AppState) -> AppResult<Vec<UserResponse>> {
    let items = repo::users::find_all(state.db()).await?;
    Ok(items.iter().map(UserResponse::from).collect())
}

pub async fn get_by_id(state: &AppState, id: u32) -> AppResult<UserResponse> {
    let item = repo::users::find_by_id(state.db(), id).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::User,
        })
    })?;
    Ok(UserResponse::from(&item))
}

pub async fn update(
    state: &AppState,
    id: u32,
    email: Option<String>,
    enabled: Option<u8>,
) -> AppResult<UserResponse> {
    let item = repo::users::update(state.db(), id, email, enabled).await?;
    let item = item.ok_or_else(|| {
        AppError::NotFoundError(Resource {
            details: vec![("id".into(), id.to_string())],
            resource_type: ResourceType::User,
        })
    })?;
    Ok(UserResponse::from(&item))
}

pub async fn create(
    state: &AppState,
    req: crate::dto::request::CreateUserRequest,
) -> AppResult<UserResponse> {
    let model = repo::users::create(state.db(), &req).await?;
    Ok(UserResponse::from(&model))
}

pub async fn delete(state: &AppState, id: u32) -> AppResult<()> {
    let ok = repo::users::delete_by_id(state.db(), id).await?;
    if ok {
        Ok(())
    } else {
        Err(crate::error::AppError::NotFoundError(
            crate::error::Resource {
                details: vec![("id".into(), id.to_string())],
                resource_type: crate::error::ResourceType::User,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::users::Model as UserModel;
    use crate::error::AppError;
    use crate::server::state::AppState;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let empty: Vec<UserModel> = vec![];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<UserModel, _, _>(vec![empty])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let err = get_by_id(&state, 123).await.err().expect("should err");
        matches!(err, AppError::NotFoundError(_));
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let empty: Vec<UserModel> = vec![];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<UserModel, _, _>(vec![empty])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let err = update(&state, 1, Some("x@y.com".into()), Some(1))
            .await
            .err()
            .expect("should err");
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
        let err = delete(&state, 77).await.err().expect("should err");
        matches!(err, AppError::NotFoundError(_));
    }

    fn mk_user(id: u32, username: &str) -> UserModel {
        use crate::entity::sea_orm_active_enums as E;
        UserModel {
            id,
            username: username.into(),
            password: "hash".into(),
            name: "Name".into(),
            email: format!("{username}@example.com"),
            phone: "".into(),
            language: None,
            enabled: 1,
            stream: E::Stream::View,
            events: E::Events::View,
            control: E::Control::View,
            monitors: E::Monitors::View,
            groups: E::Groups::View,
            devices: E::Devices::View,
            snapshots: E::Snapshots::View,
            system: E::System::View,
            max_bandwidth: None,
            token_min_expiry: 0,
            api_enabled: 1,
            home_view: "console".into(),
        }
    }

    #[tokio::test]
    async fn test_list_all_ok() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<UserModel, _, _>(vec![vec![
                mk_user(1, "alice"),
                mk_user(2, "bob"),
            ]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let out = list_all(&state).await.unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].username, "alice");
    }

    #[tokio::test]
    async fn test_get_by_id_ok() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<UserModel, _, _>(vec![vec![mk_user(10, "carol")]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let out = get_by_id(&state, 10).await.unwrap();
        assert_eq!(out.id, 10);
        assert_eq!(out.username, "carol");
    }

    #[tokio::test]
    async fn test_update_ok() {
        let initial = mk_user(5, "dave");
        let mut after = initial.clone();
        after.email = "new@example.com".into();
        after.enabled = 0;
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<UserModel, _, _>(vec![vec![initial]])
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 0,
                rows_affected: 1,
            }])
            .append_query_results::<UserModel, _, _>(vec![vec![after.clone()]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let out = update(&state, 5, Some("new@example.com".into()), Some(0))
            .await
            .unwrap();
        assert_eq!(out.email, "new@example.com");
        assert_eq!(out.enabled, 0);
    }

    #[tokio::test]
    async fn test_create_ok() {
        use crate::dto::request::users::CreateUserRequest;
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_exec_results(vec![MockExecResult {
                last_insert_id: 100,
                rows_affected: 1,
            }])
            .append_query_results::<UserModel, _, _>(vec![vec![mk_user(100, "eve")]])
            .into_connection();
        let state = AppState::for_test_with_db(db);
        let req = CreateUserRequest {
            username: "eve".into(),
            password: "pass".into(),
            name: Some("Eve".into()),
            email: "eve@example.com".into(),
            phone: None,
            enabled: Some(1),
        };
        let out = create(&state, req).await.unwrap();
        assert_eq!(out.username, "eve");
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
}
