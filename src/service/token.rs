use crate::constant::*;
use crate::dto::response::TokenResponse;
use crate::error::AppResult;
use crate::util::claim::UserClaims;


pub fn generate_tokens(
  username: String
) -> AppResult<TokenResponse> {
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
