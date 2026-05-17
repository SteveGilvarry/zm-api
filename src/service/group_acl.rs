//! Row-level group access control.
//!
//! ZoneMinder's `Groups_Permissions` table records, per user, which *groups*
//! they may see and at what level (`None` / `View` / `Edit`). Feature-level
//! RBAC ([`crate::util::authz`]) decides whether a user may touch the groups
//! API at all; this module decides *which groups* within it.
//!
//! [`resolve_groups`] turns those rows into a [`GroupScope`]. Policy mirrors
//! [`crate::service::monitor_acl`]:
//!
//! * A user with **no** `Groups_Permissions` rows is unrestricted
//!   ([`GroupScope::All`]) — default-allow, so deployments that never populate
//!   the table are unaffected.
//! * Once a user has any row, the rows become an allowlist
//!   ([`GroupScope::Restricted`]): only groups with an effective level of
//!   `View` or higher are visible. `Inherit`/`None` rows grant nothing.

use std::collections::HashMap;

use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::http::{HeaderMap, Uri};
use sea_orm::DatabaseConnection;

use crate::constant::ACCESS_TOKEN_DECODE_KEY;
use crate::entity::sea_orm_active_enums::Permission;
use crate::error::{AppError, AppResult};
use crate::repo;
use crate::server::state::AppState;
use crate::util::authz::Level;
use crate::util::claim::UserClaims;

/// The set of groups a user may access, with each group's effective level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupScope {
    /// Unrestricted — the user has no `Groups_Permissions` rows (default-allow).
    All,
    /// Restricted to an explicit allowlist of `group_id -> effective level`.
    Restricted(HashMap<u32, Level>),
}

impl GroupScope {
    /// Whether the caller may access `group_id` at `required` level or above.
    pub fn allows(&self, group_id: u32, required: Level) -> bool {
        match self {
            GroupScope::All => true,
            GroupScope::Restricted(map) => {
                map.get(&group_id).is_some_and(|level| *level >= required)
            }
        }
    }

    /// The group ids visible at `required` level, or `None` when the scope is
    /// unrestricted ([`GroupScope::All`]).
    ///
    /// `Some(vec![])` is a meaningful result: a restricted user who may see no
    /// groups at all.
    pub fn visible_ids(&self, required: Level) -> Option<Vec<u32>> {
        match self {
            GroupScope::All => None,
            GroupScope::Restricted(map) => Some(
                map.iter()
                    .filter(|(_, level)| **level >= required)
                    .map(|(id, _)| *id)
                    .collect(),
            ),
        }
    }

    /// Whether this scope restricts access at all.
    pub fn is_restricted(&self) -> bool {
        matches!(self, GroupScope::Restricted(_))
    }
}

/// Resolve the group access scope for a user from `Groups_Permissions`.
pub async fn resolve_groups(db: &DatabaseConnection, user_id: u32) -> AppResult<GroupScope> {
    let perms = repo::groups_permissions::find_by_user_id(db, user_id).await?;

    // Default-allow: a user with no permission rows is unrestricted.
    if perms.is_empty() {
        return Ok(GroupScope::All);
    }

    let mut map: HashMap<u32, Level> = HashMap::new();
    for gp in &perms {
        let level = match gp.permission {
            Permission::View => Level::View,
            Permission::Edit => Level::Edit,
            // `Inherit`/`None` grant nothing — the group stays out of the map,
            // which `allows` treats as denied.
            Permission::Inherit | Permission::None => continue,
        };
        let entry = map.entry(gp.group_id).or_insert(Level::None);
        *entry = (*entry).max(level);
    }

    Ok(GroupScope::Restricted(map))
}

/// Extract a bearer token — `Authorization: Bearer <jwt>` header, or the
/// `?token=` query parameter.
fn extract_token(headers: &HeaderMap, uri: &Uri) -> Option<String> {
    let from_header = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|token| token.trim().to_string());

    from_header.or_else(|| {
        uri.query().and_then(|query| {
            query.split('&').find_map(|pair| {
                let (key, value) = pair.split_once('=')?;
                (key == "token").then(|| value.to_string())
            })
        })
    })
}

/// Resolve the caller's group scope from a token in the given headers/URI.
async fn resolve_from_request(
    state: &AppState,
    headers: &HeaderMap,
    uri: &Uri,
) -> AppResult<GroupScope> {
    let token = extract_token(headers, uri)
        .ok_or_else(|| AppError::UnauthorizedError("Authentication required".to_string()))?;
    let claims = UserClaims::decode(&token, &ACCESS_TOKEN_DECODE_KEY)
        .map_err(|_| AppError::UnauthorizedError("Invalid token".to_string()))?
        .claims;
    resolve_groups(state.db(), claims.uid).await
}

/// Axum extractor that resolves the caller's [`GroupScope`].
///
/// Self-contained — it decodes the access token itself, so it does not depend
/// on auth-middleware ordering. The scope is resolved live from the database,
/// so group-permission changes take effect immediately.
impl FromRequestParts<AppState> for GroupScope {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        resolve_from_request(state, &parts.headers, &parts.uri).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::groups_permissions;
    use sea_orm::{DatabaseBackend, MockDatabase};

    fn grp_perm(id: u32, group_id: u32, permission: Permission) -> groups_permissions::Model {
        groups_permissions::Model {
            id,
            group_id,
            user_id: 7,
            permission,
        }
    }

    #[tokio::test]
    async fn no_rows_is_unrestricted() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<groups_permissions::Model, _, _>(vec![vec![]])
            .into_connection();
        assert_eq!(resolve_groups(&db, 7).await.unwrap(), GroupScope::All);
    }

    #[tokio::test]
    async fn grants_form_an_allowlist() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![vec![
                grp_perm(1, 10, Permission::View),
                grp_perm(2, 11, Permission::Edit),
            ]])
            .into_connection();

        let scope = resolve_groups(&db, 7).await.unwrap();
        assert!(scope.is_restricted());
        // Group 10: read but not write.
        assert!(scope.allows(10, Level::View));
        assert!(!scope.allows(10, Level::Edit));
        // Group 11: read and write.
        assert!(scope.allows(11, Level::Edit));
        // Group 99: no row at all.
        assert!(!scope.allows(99, Level::View));
    }

    #[tokio::test]
    async fn inherit_and_none_grant_nothing() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![vec![
                grp_perm(1, 10, Permission::Inherit),
                grp_perm(2, 11, Permission::None),
            ]])
            .into_connection();

        let scope = resolve_groups(&db, 7).await.unwrap();
        // The user has rows, so the scope is restricted...
        assert!(scope.is_restricted());
        // ...but neither group is visible.
        assert!(!scope.allows(10, Level::View));
        assert!(!scope.allows(11, Level::View));
    }

    #[tokio::test]
    async fn highest_level_wins_for_duplicate_rows() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![vec![
                grp_perm(1, 10, Permission::View),
                grp_perm(2, 10, Permission::Edit),
            ]])
            .into_connection();

        let scope = resolve_groups(&db, 7).await.unwrap();
        assert!(scope.allows(10, Level::Edit));
    }

    #[test]
    fn visible_ids_filters_by_required_level() {
        let mut map = HashMap::new();
        map.insert(10, Level::View);
        map.insert(11, Level::Edit);
        let scope = GroupScope::Restricted(map);

        let mut view = scope.visible_ids(Level::View).unwrap();
        view.sort_unstable();
        assert_eq!(view, vec![10, 11]);

        let edit = scope.visible_ids(Level::Edit).unwrap();
        assert_eq!(edit, vec![11]);

        assert_eq!(GroupScope::All.visible_ids(Level::View), None);
    }
}
