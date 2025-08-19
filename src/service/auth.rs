use tracing::info;
use crate::constant::*;
use crate::error::ToAppResult;
use crate::error::AppResult;
use crate::dto::*;
use crate::repo::user;
use crate::service::token;
use crate::util::password;
use crate::server::state::AppState;
use crate::util::claim::UserClaims;
use crate::dto::response::TokenResponse;



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

pub async fn login(state: &AppState, req: LoginRequest) -> AppResult<TokenResponse> {
    info!("Login user request :{req:?}.");
    let user = user::find_by_username_and_status(&state.db, &req.username, true)
        .await?
        .to_result()?;
    password::verify(req.password.clone(), user.password.clone()).await?;
    let resp = token::generate_tokens(user.username)?;
    Ok(resp)
}

pub async fn refresh_token(state: &AppState, req: RefreshTokenRequest) -> AppResult<TokenResponse> {
    let user_claims = UserClaims::decode(&req.token, &REFRESH_TOKEN_DECODE_KEY)?.claims;
    info!("Refresh token: {user_claims:?}");
    let user = user::find_by_username_and_status(&state.db, &user_claims.user, true)
        .await?
        .to_result()?;
    info!("Set new session for user: {}", user.id);
    let resp = token::generate_tokens(user.username)?;
    info!("Refresh token success: {user_claims:?}");
    Ok(resp)
}