use axum::extract::State;
use axum::Json;
use garde::Validate;
use tracing::{info, warn};

use crate::error::AppResult;
use crate::server::state::AppState;
use crate::{dto::*, service};
use crate::util::claim::UserClaimsRequest;

// Login user.
#[utoipa::path(
        post,
        request_body = LoginRequest,
        path = "/api/v3/auth/login",
        responses(
                (status = 200, description = "Success login user", body = [LoginResponse]),
                (status = 400, description = "Invalid data input", body = [AppResponseError]),
                (status = 500, description = "Internal server error", body = [AppResponseError])
        ),
        tag = "Auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<TokenResponse>> {
    info!("Login user with request: {req:?}.");
    req.validate()?;
    match service::auth::login(&state, req).await {
        Ok(resp) => {
            info!("Successfully login user: {resp:?}.");
            Ok(Json(resp))
        }
        Err(e) => {
            warn!("Unsuccessfully login user error: {e:?}.");
            Err(e)
        }
    }
}

/// Refresh token.
#[utoipa::path(
    post,
    path = "/api/v3/auth/refresh",
    responses(
        (status = 200, description = "Success get new access token and refresh token", body = TokenResponse),
        (status = 400, description = "Invalid data input", body = AppResponseError),
        (status = 401, description = "Unauthorized user", body = AppResponseError),
        (status = 500, description = "Internal server error", body = AppResponseError)
    ),
    tag = "Auth"
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> AppResult<Json<TokenResponse>> {
    info!("Refresh token with request: {req:?}.");
    match service::auth::refresh_token(&state, req).await {
        Ok(resp) => {
            info!("Success refresh token user response: {resp:?}.");
            Ok(Json(resp))
        }
        Err(e) => {
            warn!("Unsuccessfully refresh token error: {e:?}.");
            Err(e)
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v3/auth/logout",
    responses(
        (status = 200, description = "Successfully logged out", body = MessageResponse),
        (status = 401, description = "Unauthorized - Invalid or missing token", body = AppResponseError)
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Auth"
)]
pub async fn logout(State(_state): State<AppState>, request: axum::extract::Request) -> AppResult<Json<MessageResponse>> {
    // Get username from the JWT token
    let username = request.get_user_name()?;
    info!("Handling logout request for user: {}", username);
    
    // In a real implementation, you might invalidate the token in Redis or a token blacklist
    // For now, we'll just acknowledge the logout
    
    Ok(Json(MessageResponse::new("Logout successful")))
}