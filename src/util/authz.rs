//! Role-based access control (RBAC) enforcement.
//!
//! ZoneMinder's `Users` table carries per-feature permission columns
//! (`Stream`, `Events`, `Control`, `Monitors`, `Groups`, `Devices`,
//! `Snapshots`, `System`), each `None` / `View` / `Edit`. Authentication
//! alone only proves *who* the caller is — this module enforces *what* they
//! are allowed to do.
//!
//! Permissions are embedded in the access token at login time (see
//! [`crate::service::token`]) and carried in [`UserClaims::perms`]. The
//! [`protect`] middleware is self-contained: it decodes the bearer token
//! itself, so it does not depend on any other middleware running first and
//! can be applied to a router from a single place.

use axum::{
    extract::Request,
    http::Method,
    middleware::{from_fn, Next},
    response::Response,
    Router,
};
use fake::Dummy;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::constant::ACCESS_TOKEN_DECODE_KEY;
use crate::entity::sea_orm_active_enums as enums;
use crate::entity::users::Model as UserModel;
use crate::error::AppError;
use crate::server::state::AppState;
use crate::util::claim::UserClaims;
use crate::util::middleware::{extract_token_from_header, extract_token_from_query};

/// A permission level, ordered `None < View < Edit`.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Serialize,
    Deserialize,
    Dummy,
    ToSchema,
)]
pub enum Level {
    /// No access.
    #[default]
    None,
    /// Read-only access.
    View,
    /// Read-write access.
    Edit,
}

/// A snapshot of a user's per-feature permission levels, embedded in the JWT.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Serialize,
    Deserialize,
    Dummy,
    ToSchema,
)]
pub struct UserPermissions {
    pub stream: Level,
    pub events: Level,
    pub control: Level,
    pub monitors: Level,
    pub groups: Level,
    pub devices: Level,
    pub snapshots: Level,
    pub system: Level,
}

impl UserPermissions {
    /// A permission set granting full `Edit` access to every feature.
    /// Intended for tests and tooling, never for real token issuance.
    pub fn superuser() -> Self {
        Self {
            stream: Level::View, // `Stream` has no `Edit` level in ZoneMinder
            events: Level::Edit,
            control: Level::Edit,
            monitors: Level::Edit,
            groups: Level::Edit,
            devices: Level::Edit,
            snapshots: Level::Edit,
            system: Level::Edit,
        }
    }

    /// The caller's level for a given feature.
    pub fn level(&self, feature: Feature) -> Level {
        match feature {
            Feature::Stream => self.stream,
            Feature::Events => self.events,
            Feature::Control => self.control,
            Feature::Monitors => self.monitors,
            Feature::Groups => self.groups,
            Feature::Devices => self.devices,
            Feature::Snapshots => self.snapshots,
            Feature::System => self.system,
        }
    }
}

impl From<&UserModel> for UserPermissions {
    fn from(u: &UserModel) -> Self {
        Self {
            stream: match u.stream {
                enums::Stream::None => Level::None,
                enums::Stream::View => Level::View,
            },
            events: from_nve(&u.events),
            control: match u.control {
                enums::Control::None => Level::None,
                enums::Control::View => Level::View,
                enums::Control::Edit => Level::Edit,
            },
            monitors: match u.monitors {
                enums::Monitors::None => Level::None,
                enums::Monitors::View => Level::View,
                // `Create` is a stronger form of write access than `Edit`.
                enums::Monitors::Edit | enums::Monitors::Create => Level::Edit,
            },
            groups: match u.groups {
                enums::Groups::None => Level::None,
                enums::Groups::View => Level::View,
                enums::Groups::Edit => Level::Edit,
            },
            devices: match u.devices {
                enums::Devices::None => Level::None,
                enums::Devices::View => Level::View,
                enums::Devices::Edit => Level::Edit,
            },
            snapshots: match u.snapshots {
                enums::Snapshots::None => Level::None,
                enums::Snapshots::View => Level::View,
                enums::Snapshots::Edit => Level::Edit,
            },
            system: match u.system {
                enums::System::None => Level::None,
                enums::System::View => Level::View,
                enums::System::Edit => Level::Edit,
            },
        }
    }
}

fn from_nve(e: &enums::Events) -> Level {
    match e {
        enums::Events::None => Level::None,
        enums::Events::View => Level::View,
        enums::Events::Edit => Level::Edit,
    }
}

/// A ZoneMinder permission feature/category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    Stream,
    Events,
    Control,
    Monitors,
    Groups,
    Devices,
    Snapshots,
    System,
}

/// The permission level a request needs, derived from the HTTP method:
/// safe (read) methods need `View`, mutating methods need `Edit`.
///
/// `Stream` has no `Edit` level in ZoneMinder, so it always requires `View`.
fn required_level(feature: Feature, method: &Method) -> Level {
    if feature == Feature::Stream {
        return Level::View;
    }
    match *method {
        Method::GET | Method::HEAD | Method::OPTIONS => Level::View,
        _ => Level::Edit,
    }
}

async fn enforce(feature: Feature, request: Request, next: Next) -> Result<Response, AppError> {
    // Self-contained: decode the bearer token (header or `?token=` for media
    // elements) rather than relying on auth middleware ordering.
    let token = extract_token_from_header(&request)
        .or_else(|| extract_token_from_query(&request))
        .ok_or_else(|| AppError::UnauthorizedError("Authentication required".to_string()))?;

    let claims = UserClaims::decode(&token, &ACCESS_TOKEN_DECODE_KEY)
        .map_err(|_| AppError::UnauthorizedError("Invalid token".to_string()))?
        .claims;

    let required = required_level(feature, request.method());
    let granted = claims.perms.level(feature);

    if granted >= required {
        Ok(next.run(request).await)
    } else {
        Err(AppError::PermissionDeniedError(format!(
            "{:?} access to {:?} required",
            required, feature
        )))
    }
}

/// Wrap a router so every request is checked against the given [`Feature`].
///
/// The required level is method-derived (read → `View`, write → `Edit`).
/// Apply this to routers whose endpoints all belong to one feature.
pub fn protect(router: Router<AppState>, feature: Feature) -> Router<AppState> {
    router.layer(from_fn(move |req: Request, next: Next| {
        enforce(feature, req, next)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_ordering() {
        assert!(Level::None < Level::View);
        assert!(Level::View < Level::Edit);
    }

    #[test]
    fn required_level_is_method_aware() {
        assert_eq!(required_level(Feature::Monitors, &Method::GET), Level::View);
        assert_eq!(
            required_level(Feature::Monitors, &Method::POST),
            Level::Edit
        );
        assert_eq!(
            required_level(Feature::Monitors, &Method::DELETE),
            Level::Edit
        );
        // Stream never requires Edit.
        assert_eq!(required_level(Feature::Stream, &Method::POST), Level::View);
    }

    #[test]
    fn superuser_grants_every_feature() {
        let p = UserPermissions::superuser();
        for f in [
            Feature::Events,
            Feature::Control,
            Feature::Monitors,
            Feature::Groups,
            Feature::Devices,
            Feature::Snapshots,
            Feature::System,
        ] {
            assert_eq!(p.level(f), Level::Edit);
        }
        assert_eq!(p.level(Feature::Stream), Level::View);
    }

    #[test]
    fn default_permissions_grant_nothing() {
        let p = UserPermissions::default();
        assert_eq!(p.level(Feature::Monitors), Level::None);
        assert_eq!(p.level(Feature::System), Level::None);
    }
}
