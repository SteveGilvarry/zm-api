//! `/api/v3/notifications`: push token registrations (#175). Any logged-in
//! user manages their own; a System editor sees and manages all of them.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use garde::Validate;

use crate::dto::request::notifications::{CreateNotificationRequest, UpdateNotificationRequest};
use crate::dto::response::notifications::NotificationResponse;
use crate::error::{AppError, AppResponseError, AppResult};
use crate::server::state::AppState;
use crate::service::notifications::{self as svc, Caller};
use crate::util::authz::{Feature, Level};
use crate::util::claim::UserClaims;

fn caller(claims: &UserClaims) -> Caller {
    Caller {
        uid: claims.uid,
        admin: claims.perms.level(Feature::System) >= Level::Edit,
    }
}

/// List push registrations
///
/// The caller's own; every user's for a System editor.
#[utoipa::path(
    get,
    path = "/api/v3/notifications",
    operation_id = "listNotifications",
    responses((status = 200, description = "Push registrations", body = Vec<NotificationResponse>)),
    tag = "Notifications",
    security(("jwt" = []))
)]
pub async fn list_notifications(
    State(state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
) -> AppResult<Json<Vec<NotificationResponse>>> {
    Ok(Json(svc::list(&state, caller(&claims)).await?))
}

/// Get one push registration
#[utoipa::path(
    get,
    path = "/api/v3/notifications/{id}",
    operation_id = "getNotification",
    params(("id" = u32, Path, description = "Notification ID")),
    responses(
        (status = 200, description = "Push registration", body = NotificationResponse),
        (status = 404, description = "Not found, or another user's", body = AppResponseError)
    ),
    tag = "Notifications",
    security(("jwt" = []))
)]
pub async fn get_notification(
    State(state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
    Path(id): Path<u32>,
) -> AppResult<Json<NotificationResponse>> {
    Ok(Json(svc::get(&state, caller(&claims), id).await?))
}

/// Register a push token
///
/// A token already registered to the caller is updated in place (200);
/// a new one is created (201). Another user's token is a 403.
#[utoipa::path(
    post,
    path = "/api/v3/notifications",
    operation_id = "registerNotification",
    request_body = CreateNotificationRequest,
    responses(
        (status = 201, description = "Registered", body = NotificationResponse),
        (status = 200, description = "Already registered; updated", body = NotificationResponse),
        (status = 403, description = "The token belongs to another user", body = AppResponseError)
    ),
    tag = "Notifications",
    security(("jwt" = []))
)]
pub async fn create_notification(
    State(state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
    Json(req): Json<CreateNotificationRequest>,
) -> AppResult<(StatusCode, Json<NotificationResponse>)> {
    req.validate().map_err(AppError::InvalidInputError)?;
    let (created, body) = svc::upsert(&state, caller(&claims), req).await?;
    let status = if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(body)))
}

/// Change a push registration
///
/// Typically `last_notified_at` and `badge_count` after a push.
#[utoipa::path(
    patch,
    path = "/api/v3/notifications/{id}",
    operation_id = "updateNotification",
    params(("id" = u32, Path, description = "Notification ID")),
    request_body = UpdateNotificationRequest,
    responses(
        (status = 200, description = "Updated", body = NotificationResponse),
        (status = 404, description = "Not found, or another user's", body = AppResponseError),
        (status = 409, description = "The new token is already registered", body = AppResponseError)
    ),
    tag = "Notifications",
    security(("jwt" = []))
)]
pub async fn update_notification(
    State(state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
    Path(id): Path<u32>,
    Json(req): Json<UpdateNotificationRequest>,
) -> AppResult<Json<NotificationResponse>> {
    req.validate().map_err(AppError::InvalidInputError)?;
    Ok(Json(svc::update(&state, caller(&claims), id, req).await?))
}

/// Remove a push registration
#[utoipa::path(
    delete,
    path = "/api/v3/notifications/{id}",
    operation_id = "deleteNotification",
    params(("id" = u32, Path, description = "Notification ID")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Not found, or another user's", body = AppResponseError)
    ),
    tag = "Notifications",
    security(("jwt" = []))
)]
pub async fn delete_notification(
    State(state): State<AppState>,
    Extension(claims): Extension<UserClaims>,
    Path(id): Path<u32>,
) -> AppResult<StatusCode> {
    svc::delete(&state, caller(&claims), id).await?;
    Ok(StatusCode::NO_CONTENT)
}
