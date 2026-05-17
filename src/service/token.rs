#![allow(clippy::result_large_err)]
use crate::constant::*;
use crate::dto::response::TokenResponse;
use crate::error::AppResult;
use crate::util::authz::UserPermissions;
use crate::util::claim::UserClaims;

/// Issue an access + refresh token pair for a user.
///
/// `user_id` keys row-level monitor ACLs; `perms` is the RBAC permission
/// snapshot. Both are embedded in the token so authorization needs no
/// database round-trip for feature-level checks.
pub fn generate_tokens(
    username: String,
    user_id: u32,
    perms: UserPermissions,
) -> AppResult<TokenResponse> {
    let access_token = UserClaims::new(EXPIRE_BEARER_TOKEN_SECS, username.clone(), user_id, perms)
        .encode(&ACCESS_TOKEN_ENCODE_KEY)?;
    let refresh_token = UserClaims::new(EXPIRE_REFRESH_TOKEN_SECS, username, user_id, perms)
        .encode(&REFRESH_TOKEN_ENCODE_KEY)?;
    Ok(TokenResponse::new(
        access_token,
        refresh_token,
        EXPIRE_BEARER_TOKEN_SECS.as_secs(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_tokens_returns_bearer_tokens() {
        let out = generate_tokens("tester".into(), 1, UserPermissions::superuser()).unwrap();
        assert_eq!(out.token_type, crate::constant::BEARER);
        assert!(!out.access_token.is_empty());
        assert!(!out.refresh_token.is_empty());
        assert!(out.expire_in > 0);
    }
}
