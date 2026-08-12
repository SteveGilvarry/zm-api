use axum::extract::State;
use axum::Json;
use garde::Validate;
use tracing::{info, warn};

use crate::dto::request::{ChangePasswordRequest, LoginRequest, RefreshTokenRequest};
use crate::dto::response::{MessageResponse, TokenResponse, UserResponse};
use crate::error::AppResponseError;
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::server::state::AppState;
use crate::service;
use crate::util::claim::{UserClaims, UserClaimsRequest};
use axum::Extension;

// Login user.
#[utoipa::path(
        post,
        request_body = LoginRequest,
        path = "/api/v3/auth/login",
        responses(
                (status = 200, description = "Success login user", body = TokenResponse),
                (status = 400, description = "Invalid data input", body = AppResponseError),
                (status = 500, description = "Internal server error", body = AppResponseError)
        ),
        tag = "Auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<TokenResponse>> {
    info!("Login attempt for user: {}.", req.username);
    req.validate()?;
    let username = req.username.clone();
    match service::auth::login(&state, req).await {
        Ok(resp) => {
            info!("Successfully logged in user: {username}.");
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
    info!("Refresh token request received.");
    req.validate()?;
    match service::auth::refresh_token(&state, req).await {
        Ok(resp) => {
            info!("Successfully refreshed token.");
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
pub async fn logout(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> AppResult<Json<MessageResponse>> {
    let claims = request.get_user_claims()?;
    info!("Handling logout request for user: {}", claims.user);

    // Raise the user's TokenMinExpiry floor so every outstanding access and
    // refresh token (including this one) is revoked server-side.
    service::auth::logout(&state, claims.uid).await?;

    Ok(Json(MessageResponse::new("Logout successful")))
}

/// Return the authenticated user's own account.
#[utoipa::path(
    get,
    path = "/api/v3/me",
    responses(
        (status = 200, description = "The authenticated user's account", body = UserResponse),
        (status = 401, description = "Unauthorized - Invalid or missing token", body = AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Auth"
)]
pub async fn me(
    State(state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
) -> AppResult<Json<UserResponse>> {
    let user = crate::repo::users::find_by_id(&state.db, claims.uid)
        .await?
        .ok_or_else(|| {
            AppError::NotFoundError(Resource {
                details: vec![("id".to_string(), claims.uid.to_string())],
                resource_type: ResourceType::User,
            })
        })?;
    Ok(Json(UserResponse::from(&user)))
}

/// Change the authenticated user's own password.
///
/// Requires the current password; on success every existing token for the
/// user is revoked, so the client must re-authenticate.
#[utoipa::path(
    put,
    path = "/api/v3/me/password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed; existing tokens revoked", body = MessageResponse),
        (status = 400, description = "Invalid data input", body = AppResponseError),
        (status = 401, description = "Unauthorized or current password incorrect", body = AppResponseError)
    ),
    security(("jwt" = [])),
    tag = "Auth"
)]
pub async fn change_password(
    State(state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<Json<MessageResponse>> {
    req.validate()?;

    service::auth::change_password(&state, claims.uid, req.current_password, req.new_password)
        .await?;

    Ok(Json(MessageResponse::new(
        "Password changed; please sign in again",
    )))
}
