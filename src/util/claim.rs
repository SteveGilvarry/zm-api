#![allow(clippy::result_large_err)]
use std::time::Duration;

use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::RequestPartsExt;
use chrono::Utc;
use fake::Dummy;
use jsonwebtoken::Header;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, TokenData, Validation};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;

use crate::constant::{ACCESS_TOKEN_DECODE_KEY, REFRESH_TOKEN_DECODE_KEY};
use crate::error::{AppError, AppResult};
use crate::util::authz::UserPermissions;

pub static DECODE_HEADER: Lazy<Validation> = Lazy::new(|| Validation::new(Algorithm::RS256));
pub static ENCODE_HEADER: Lazy<Header> = Lazy::new(|| Header::new(Algorithm::RS256));

/// Which of the two token kinds a JWT is. Access and refresh tokens are
/// otherwise structurally identical `UserClaims`, separated only by the key
/// pair that signs them; `typ` makes the distinction explicit so a
/// misconfiguration that pointed both key paths at the same pair could not
/// silently let a refresh token act as an access token (or vice versa).
#[derive(
    Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Dummy, ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Dummy, ToSchema)]
pub struct UserClaims {
    // issued at
    pub iat: i64,
    // expiration
    pub exp: i64,
    // token kind — asserted on decode so access and refresh tokens are not
    // interchangeable even if the signing keys were ever misconfigured.
    pub typ: TokenType,
    // username
    pub user: String,
    // numeric user id, used to resolve row-level monitor ACLs. Required at
    // the protocol level: a token without `uid` fails to deserialise and the
    // request is rejected. (The earlier `#[serde(default)]` was speculative
    // backward-compat for tokens issued before this field existed — but the
    // field was added together with the ACL machinery, so no such tokens
    // could ever have exploited the missing-uid default-allow path.)
    pub uid: u32,
    // per-feature permission snapshot (RBAC). Defaulted for backward
    // compatibility with tokens issued before RBAC existed.
    #[serde(default)]
    pub perms: UserPermissions,
}

impl UserClaims {
    pub fn new(
        duration: Duration,
        username: String,
        user_id: u32,
        perms: UserPermissions,
        typ: TokenType,
    ) -> Self {
        let now = Utc::now().timestamp();
        Self {
            iat: now,
            exp: now + (duration.as_secs() as i64),
            typ,
            user: username,
            uid: user_id,
            perms,
        }
    }

    pub fn decode(
        token: &str,
        key: &DecodingKey,
    ) -> Result<TokenData<Self>, jsonwebtoken::errors::Error> {
        jsonwebtoken::decode::<UserClaims>(token, key, &DECODE_HEADER)
    }

    /// Decode and validate an **access** token against the access key, then
    /// assert its `typ` is [`TokenType::Access`]. A refresh token presented
    /// here is rejected as `InvalidToken`.
    pub fn decode_access(token: &str) -> Result<TokenData<Self>, jsonwebtoken::errors::Error> {
        let data = Self::decode(token, &ACCESS_TOKEN_DECODE_KEY)?;
        if data.claims.typ != TokenType::Access {
            return Err(jsonwebtoken::errors::ErrorKind::InvalidToken.into());
        }
        Ok(data)
    }

    /// Decode and validate a **refresh** token against the refresh key, then
    /// assert its `typ` is [`TokenType::Refresh`].
    pub fn decode_refresh(token: &str) -> Result<TokenData<Self>, jsonwebtoken::errors::Error> {
        let data = Self::decode(token, &REFRESH_TOKEN_DECODE_KEY)?;
        if data.claims.typ != TokenType::Refresh {
            return Err(jsonwebtoken::errors::ErrorKind::InvalidToken.into());
        }
        Ok(data)
    }

    pub fn encode(&self, key: &EncodingKey) -> Result<String, jsonwebtoken::errors::Error> {
        jsonwebtoken::encode(&ENCODE_HEADER, self, key)
    }
}

pub trait UserClaimsRequest {
    fn get_user_name(&self) -> AppResult<String>;
    fn get_user_claims(&self) -> AppResult<UserClaims>;
}

impl UserClaimsRequest for axum::extract::Request {
    fn get_user_name(&self) -> AppResult<String> {
        self.extensions()
            .get::<UserClaims>()
            .map(|u| u.user.clone())
            .ok_or_else(|| AppError::UnauthorizedError("User Must Login".to_string()))
    }

    fn get_user_claims(&self) -> AppResult<UserClaims> {
        self.extensions()
            .get::<UserClaims>()
            .cloned()
            .ok_or_else(|| AppError::UnauthorizedError("User Must Login".to_string()))
    }
}

impl<S> FromRequestParts<S> for UserClaims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await?;
        let user_claims = UserClaims::decode_access(bearer.token())?.claims;
        Ok(user_claims)
    }
}

#[cfg(test)]
mod tests {
    use crate::util::key::RsaPairKey;
    use fake::{Fake, Faker};

    use super::*;

    #[test]
    fn test_user_claims() {
        let username: String = Faker.fake();
        let pair_key = RsaPairKey::new(2048).unwrap();
        let claims = UserClaims::new(
            Duration::from_secs(100),
            username,
            42,
            UserPermissions::superuser(),
            TokenType::Access,
        );
        // println!(
        //     "private key: {}",
        //     String::from_utf8(pair_key.private_key.clone()).unwrap()
        // );
        // println!(
        //     "public key: {}",
        //     String::from_utf8(pair_key.public_key.clone()).unwrap()
        // );
        let token = claims
            .encode(&EncodingKey::from_rsa_pem(&pair_key.private_key).unwrap())
            .unwrap();
        let actual_claims = UserClaims::decode(
            &token,
            &DecodingKey::from_rsa_pem(&pair_key.public_key).unwrap(),
        )
        .unwrap()
        .claims;
        assert_eq!(actual_claims, claims)
    }
}
