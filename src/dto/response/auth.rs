use crate::constant::BEARER;
use fake::Dummy;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema, Dummy)]
#[serde(tag = "type")]
pub enum LoginResponse {
    Token(TokenResponse),
    Code { message: String, expire_in: u64 },
}

impl From<TokenResponse> for LoginResponse {
    fn from(value: TokenResponse) -> Self {
        LoginResponse::Token(value)
    }
}

#[derive(Serialize, Deserialize, ToSchema, Dummy, Clone)]
pub struct TokenResponse {
    pub token_type: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expire_in: u64,
}

// Manual Debug: both tokens are live credentials and must never reach logs.
impl std::fmt::Debug for TokenResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenResponse")
            .field("token_type", &self.token_type)
            .field("access_token", &"[REDACTED]")
            .field("refresh_token", &"[REDACTED]")
            .field("expire_in", &self.expire_in)
            .finish()
    }
}

impl TokenResponse {
    pub fn new(access_token: String, refresh_token: String, expire_in: u64) -> Self {
        Self {
            token_type: BEARER.to_string(),
            access_token,
            refresh_token,
            expire_in,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_response_debug_redacts_tokens() {
        let resp = TokenResponse::new(
            "eyJhbGciOiJSUzI1NiJ9.access.secret".to_string(),
            "eyJhbGciOiJSUzI1NiJ9.refresh.secret".to_string(),
            600,
        );
        let out = format!("{resp:?}");
        assert!(!out.contains("secret"));
        assert!(!out.contains("eyJhbGciOiJSUzI1NiJ9"));
        assert!(out.contains("[REDACTED]"));
        // LoginResponse wraps TokenResponse; its derived Debug must inherit the redaction.
        let wrapped = format!("{:?}", LoginResponse::Token(resp));
        assert!(!wrapped.contains("secret"));
    }
}

/// The authenticated caller's account plus the metadata carried by their
/// current access token, so a client can show "signed in as / expires at"
/// without decoding the JWT itself (GH #37).
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MeResponse {
    /// The caller's user record (permissions, profile).
    pub user: crate::dto::response::users::UserResponse,
    /// Token issued-at, unix seconds.
    pub issued_at: i64,
    /// Token expiry, unix seconds.
    pub expires_at: i64,
    /// Token kind: `access` or `refresh` (always `access` here).
    pub token_type: String,
}
