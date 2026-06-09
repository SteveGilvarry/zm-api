#![allow(clippy::result_large_err)]
use crate::constant::*;
use crate::dto::request::{LoginRequest, RefreshTokenRequest};
use crate::dto::response::TokenResponse;
use crate::error::AppError;
use crate::error::AppResult;
use crate::error::ToAppResult;
use crate::repo::users as user;
use crate::server::state::AppState;
use crate::service::token;
use crate::util::authz::UserPermissions;
use crate::util::claim::UserClaims;
use crate::util::password;
use tracing::info;

/// Single error returned for any login failure. Identical wording, status,
/// and timing on "no such user", "user not enabled", "API access not
/// granted", and "wrong password" — see `verify_existing_or_dummy` in
/// `util::password` for the timing side of the contract.
fn invalid_credentials() -> AppError {
    AppError::UnauthorizedError("Invalid username or password".to_string())
}

pub async fn login(state: &AppState, req: LoginRequest) -> AppResult<TokenResponse> {
    info!("Login attempt for user: {}", req.username);
    let user_opt = user::find_by_username_and_status(&state.db, &req.username, true).await?;
    // Pull the hash out before consuming user_opt — verify spends bcrypt's
    // wall-clock cost on the dummy hash when the user lookup missed, so the
    // "no such user" path doesn't return faster than "wrong password".
    let user_hash = user_opt.as_ref().map(|u| u.password.clone());
    let password_ok = password::verify_existing_or_dummy(req.password, user_hash).await;

    let user = match (user_opt, password_ok) {
        (Some(u), true) => u,
        _ => return Err(invalid_credentials()),
    };

    let perms = UserPermissions::from(&user);
    let resp = token::generate_tokens(user.username, user.id, perms)?;
    Ok(resp)
}

pub async fn refresh_token(state: &AppState, req: RefreshTokenRequest) -> AppResult<TokenResponse> {
    let user_claims = UserClaims::decode(&req.token, &REFRESH_TOKEN_DECODE_KEY)?.claims;
    info!("Refresh token: {user_claims:?}");
    let user = user::find_by_username_and_status(&state.db, &user_claims.user, true)
        .await?
        .to_result()?;
    info!("Set new session for user: {}", user.id);
    // Re-read permissions from the user row so a refresh picks up any
    // permission changes made since the previous token was issued.
    let perms = UserPermissions::from(&user);
    let resp = token::generate_tokens(user.username, user.id, perms)?;
    info!("Refresh token success: {user_claims:?}");
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

    /// Both "wrong password" and "no such user" must surface the same
    /// generic 401-flavoured error — no leak of which side failed.
    #[tokio::test]
    async fn test_login_wrong_password_returns_unauthorized() {
        let hashed_other = crate::util::password::hash("different".to_string())
            .await
            .unwrap();
        let user_row = mk_user("bob", &hashed_other);
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<crate::entity::users::Model, _, _>(vec![vec![user_row]])
            .into_connection();
        let state = AppState::for_test_with_db(db);

        let err = login(
            &state,
            LoginRequest {
                username: "bob".into(),
                password: "wrong".into(),
            },
        )
        .await
        .expect_err("should fail");
        assert!(
            matches!(err, AppError::UnauthorizedError(_)),
            "wrong-password should surface UnauthorizedError, got {err:?}"
        );
    }

    #[tokio::test]
    async fn test_login_unknown_user_returns_unauthorized() {
        let empty: Vec<crate::entity::users::Model> = vec![];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<crate::entity::users::Model, _, _>(vec![empty])
            .into_connection();
        let state = AppState::for_test_with_db(db);

        let err = login(
            &state,
            LoginRequest {
                username: "nobody".into(),
                password: "x".into(),
            },
        )
        .await
        .expect_err("should fail");
        assert!(
            matches!(err, AppError::UnauthorizedError(_)),
            "missing-user should surface UnauthorizedError (no leak vs wrong-pw), got {err:?}"
        );
    }

    /// `APIEnabled = 0` is meant to disable API access while leaving web-UI
    /// login intact. The repo filters on the column, so a disabled-API
    /// user looks like a missing user to the auth path — same unified
    /// error.
    #[tokio::test]
    async fn test_login_api_disabled_user_returns_unauthorized() {
        // Empty result set models how `find_by_username_and_status` will
        // behave once the `ApiEnabled = 1` filter is applied: a row with
        // ApiEnabled = 0 doesn't match, so the query returns no rows.
        let empty: Vec<crate::entity::users::Model> = vec![];
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<crate::entity::users::Model, _, _>(vec![empty])
            .into_connection();
        let state = AppState::for_test_with_db(db);

        let err = login(
            &state,
            LoginRequest {
                username: "carol".into(),
                password: "anything".into(),
            },
        )
        .await
        .expect_err("api-disabled user must not authenticate");
        assert!(
            matches!(err, AppError::UnauthorizedError(_)),
            "api-disabled user should surface UnauthorizedError, got {err:?}"
        );
    }
}
