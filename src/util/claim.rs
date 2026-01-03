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

use crate::constant::ACCESS_TOKEN_DECODE_KEY;
use crate::error::{AppError, AppResult};

pub static DECODE_HEADER: Lazy<Validation> = Lazy::new(|| Validation::new(Algorithm::RS256));
pub static ENCODE_HEADER: Lazy<Header> = Lazy::new(|| Header::new(Algorithm::RS256));

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Dummy, ToSchema)]
pub struct UserClaims {
    // issued at
    pub iat: i64,
    // expiration
    pub exp: i64,
    // username
    pub user: String,
}

impl UserClaims {
    pub fn new(duration: Duration, username: String) -> Self {
        let now = Utc::now().timestamp();
        Self {
            iat: now,
            exp: now + (duration.as_secs() as i64),
            user: username,
        }
    }

    pub fn decode(
        token: &str,
        key: &DecodingKey,
    ) -> Result<TokenData<Self>, jsonwebtoken::errors::Error> {
        jsonwebtoken::decode::<UserClaims>(token, key, &DECODE_HEADER)
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
        let user_claims = UserClaims::decode(bearer.token(), &ACCESS_TOKEN_DECODE_KEY)?.claims;
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
        let claims = UserClaims::new(Duration::from_secs(100), username);
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
