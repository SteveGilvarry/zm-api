#![allow(clippy::result_large_err)]
use crate::constant::*;
use crate::dto::request::{LoginRequest, RefreshTokenRequest};
use crate::dto::response::TokenResponse;
use crate::error::AppResult;
use crate::error::ToAppResult;
use crate::repo::users as user;
use crate::server::state::AppState;
use crate::service::token;
use crate::util::claim::UserClaims;
use crate::util::password;
use tracing::info;

pub fn generate_tokens(username: String) -> AppResult<TokenResponse> {
    let access_token = UserClaims::new(EXPIRE_BEARER_TOKEN_SECS, username.to_string())
        .encode(&ACCESS_TOKEN_ENCODE_KEY)?;
    let refresh_token = UserClaims::new(EXPIRE_REFRESH_TOKEN_SECS, username.to_string())
        .encode(&REFRESH_TOKEN_ENCODE_KEY)?;
    Ok(TokenResponse::new(
        access_token,
        refresh_token,
        EXPIRE_BEARER_TOKEN_SECS.as_secs(),
    ))
}

pub async fn login(state: &AppState, req: LoginRequest) -> AppResult<TokenResponse> {
    info!("Login user request for: {}", req.username);
    let user = user::find_by_username_and_status(&state.db, &req.username, true)
        .await?
        .to_result()?;
    password::verify(req.password.clone(), user.password.clone()).await?;
    let resp = token::generate_tokens(user.username)?;
    Ok(resp)
}

pub async fn refresh_token(state: &AppState, req: RefreshTokenRequest) -> AppResult<TokenResponse> {
    let user_claims = UserClaims::decode(&req.token, &REFRESH_TOKEN_DECODE_KEY)?.claims;
    info!("Refresh token for user: {}", user_claims.user);
    let user = user::find_by_username_and_status(&state.db, &user_claims.user, true)
        .await?
        .to_result()?;
    info!("Set new session for user: {}", user.id);
    let resp = token::generate_tokens(user.username)?;
    info!("Refresh token success for user: {}", user_claims.user);
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;
    use crate::server::state::AppState;
    use sea_orm::{DatabaseBackend, MockDatabase};

    fn mk_user(username: &str, hashed: &str) -> crate::entity::users::Model {
        use crate::entity::sea_orm_active_enums as E;
        crate::entity::users::Model {
            id: 1,
            username: username.into(),
            password: hashed.into(),
            name: "Name".into(),
            email: "a@b.com".into(),
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
    async fn test_login_success() {
        // Prepare a hashed password and DB returning that user
        let plain = "secret".to_string();
        let hashed = crate::util::password::hash(plain.clone()).await.unwrap();
        let user_row = mk_user("alice", &hashed);
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<crate::entity::users::Model, _, _>(vec![vec![user_row]])
            .into_connection();
        let state = AppState::for_test_with_db(db);

        let req = LoginRequest {
            username: "alice".into(),
            password: plain,
        };
        let resp = login(&state, req).await.unwrap();
        assert!(!resp.access_token.is_empty());
        assert!(!resp.refresh_token.is_empty());
    }

    #[tokio::test]
    async fn test_login_invalid_password() {
        std::env::set_var("NO_PROXY", "*");
        let hashed_other = crate::util::password::hash("different".to_string())
            .await
            .unwrap();
        let user_row = mk_user("bob", &hashed_other);
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<crate::entity::users::Model, _, _>(vec![vec![user_row]])
            .into_connection();
        let state = AppState::for_test_with_db(db);

        let req = LoginRequest {
            username: "bob".into(),
            password: "wrong".into(),
        };
        let err = login(&state, req).await.expect_err("should fail");
        matches!(err, AppError::InvalidInputError(_));
    }

    #[tokio::test]
    async fn test_login_user_not_found() {
        let empty: Vec<crate::entity::users::Model> = vec![];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<crate::entity::users::Model, _, _>(vec![empty])
            .into_connection();
        let state = AppState::for_test_with_db(db);

        let req = LoginRequest {
            username: "nobody".into(),
            password: "x".into(),
        };
        let err = login(&state, req).await.expect_err("should fail");
        matches!(err, AppError::NotFoundError(_));
    }
}
