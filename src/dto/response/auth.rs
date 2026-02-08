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
