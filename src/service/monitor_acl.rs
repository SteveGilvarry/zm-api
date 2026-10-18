//! Row-level monitor access control.
//!
//! ZoneMinder restricts *which monitors* a user may see via two tables:
//! `Monitors_Permissions` (direct per-monitor grants) and `Groups_Permissions`
//! + `Groups_Monitors` (per-group grants the monitor inherits).
//!
//! [`resolve`] turns those rows into a [`MonitorScope`] for a user. Policy:
//!
//! * A user with **no** rows in either table is unrestricted ([`MonitorScope::All`])
//!   — default-allow, so deployments that never populate permissions are
//!   unaffected. Feature-level RBAC ([`crate::util::authz`]) still applies.
//! * Once a user has any row, the rows become an allowlist
//!   ([`MonitorScope::Restricted`]): only monitors with an effective level of
//!   `View` or higher are visible.
//! * A direct `Monitors_Permissions` row overrides the group-derived level —
//!   `View`/`Edit` set it, `None` is an explicit deny, `Inherit` keeps the
//!   group result.

use std::collections::HashMap;

use axum::extract::{FromRequestParts, RawPathParams, Request, State};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::http::{HeaderMap, Method, Uri};
use axum::middleware::Next;
use axum::response::Response;
use sea_orm::DatabaseConnection;

use crate::entity::sea_orm_active_enums::Permission;
use crate::error::{AppError, AppResult, Resource, ResourceType};
use crate::repo;
use crate::server::state::AppState;
use crate::util::authz::Level;
use crate::util::claim::UserClaims;

/// The set of monitors a user may access, with each monitor's effective level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonitorScope {
    /// Unrestricted — the user has no permission rows (default-allow).
    All,
    /// Restricted to an explicit allowlist of `monitor_id -> effective level`.
    Restricted(HashMap<u32, Level>),
}

impl MonitorScope {
    /// Whether the caller may access `monitor_id` at `required` level or above.
    pub fn allows(&self, monitor_id: u32, required: Level) -> bool {
        match self {
            MonitorScope::All => true,
            MonitorScope::Restricted(map) => {
                map.get(&monitor_id).is_some_and(|level| *level >= required)
            }
        }
    }

    /// Like [`Self::allows`] for a resource whose monitor link is optional.
    /// A `None` monitor id (a row not tied to any monitor) is visible only to
    /// unrestricted callers.
    pub fn allows_opt(&self, monitor_id: Option<u32>, required: Level) -> bool {
        match monitor_id {
            Some(id) => self.allows(id, required),
            None => !self.is_restricted(),
        }
    }

    /// The monitor ids visible at `required` level, or `None` when the scope
    /// is unrestricted ([`MonitorScope::All`]).
    ///
    /// `Some(vec![])` is a meaningful result: a restricted user who may see no
    /// monitors at all.
    pub fn visible_ids(&self, required: Level) -> Option<Vec<u32>> {
        match self {
            MonitorScope::All => None,
            MonitorScope::Restricted(map) => Some(
                map.iter()
                    .filter(|(_, level)| **level >= required)
                    .map(|(id, _)| *id)
                    .collect(),
            ),
        }
    }

    /// Whether this scope restricts access at all.
    pub fn is_restricted(&self) -> bool {
        matches!(self, MonitorScope::Restricted(_))
    }
}

/// The permission rows a user's monitor scope is built from: their own, and
/// their role's (ZoneMinder 1.39), each as per-monitor grants and per-group
/// grants, plus the members of every group named.
#[derive(Debug, Default)]
pub(crate) struct AclRows {
    pub user_monitors: Vec<(u32, Permission)>,
    pub user_groups: Vec<(u32, Permission)>,
    pub role_monitors: Vec<(u32, Permission)>,
    pub role_groups: Vec<(u32, Permission)>,
    /// `group_id -> monitor ids`.
    pub group_members: HashMap<u32, Vec<u32>>,
}

/// Build the scope from the rows. Default-allow: no rows at all is
/// unrestricted. Otherwise the monitors named are an allowlist, decided by
/// layers that each override the ones before for the monitors they name:
/// the role's group grants, the role's per-monitor grants, the user's group
/// grants, the user's per-monitor grants. ZoneMinder consults the user's own
/// grants before the role's (`Monitor::canView`), hence the order.
pub(crate) fn scope_from(rows: &AclRows) -> MonitorScope {
    if rows.user_monitors.is_empty()
        && rows.user_groups.is_empty()
        && rows.role_monitors.is_empty()
        && rows.role_groups.is_empty()
    {
        return MonitorScope::All;
    }

    let mut map: HashMap<u32, Level> = HashMap::new();
    for (groups, monitors) in [
        (&rows.role_groups, &rows.role_monitors),
        (&rows.user_groups, &rows.user_monitors),
    ] {
        // Effective level per group (`Inherit`/`None` at group level grant
        // nothing), spread to the group's monitors at the highest level.
        let mut via_groups: HashMap<u32, Level> = HashMap::new();
        for (group_id, permission) in groups {
            let level = match permission {
                Permission::View => Level::View,
                Permission::Edit => Level::Edit,
                Permission::Inherit | Permission::None => continue,
            };
            for &monitor_id in rows.group_members.get(group_id).into_iter().flatten() {
                let entry = via_groups.entry(monitor_id).or_insert(Level::None);
                *entry = (*entry).max(level);
            }
        }
        map.extend(via_groups);

        // Per-monitor grants override the group-derived level.
        for (monitor_id, permission) in monitors {
            let level = match permission {
                Permission::Inherit => continue,
                Permission::None => Level::None,
                Permission::View => Level::View,
                Permission::Edit => Level::Edit,
            };
            map.insert(*monitor_id, level);
        }
    }

    MonitorScope::Restricted(map)
}

/// Resolve the monitor access scope for a user from the permission tables,
/// their role's included (#143).
pub async fn resolve(db: &DatabaseConnection, user_id: u32) -> AppResult<MonitorScope> {
    use crate::entity::{role_groups_permissions, role_monitors_permissions};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let direct = repo::monitors_permissions::find_by_user_id(db, user_id).await?;
    let group_perms = repo::groups_permissions::find_by_user_id(db, user_id).await?;
    let role_id = repo::users::find_by_id(db, user_id)
        .await?
        .and_then(|u| u.role_id);
    let (role_monitors, role_groups) = match role_id {
        Some(role_id) => (
            role_monitors_permissions::Entity::find()
                .filter(role_monitors_permissions::Column::RoleId.eq(role_id))
                .all(db)
                .await?,
            role_groups_permissions::Entity::find()
                .filter(role_groups_permissions::Column::RoleId.eq(role_id))
                .all(db)
                .await?,
        ),
        None => (Vec::new(), Vec::new()),
    };

    let mut rows = AclRows {
        user_monitors: direct
            .iter()
            .map(|p| (p.monitor_id, p.permission.clone()))
            .collect(),
        user_groups: group_perms
            .iter()
            .map(|p| (p.group_id, p.permission.clone()))
            .collect(),
        role_monitors: role_monitors
            .iter()
            .map(|p| (p.monitor_id, p.permission.clone()))
            .collect(),
        role_groups: role_groups
            .iter()
            .map(|p| (p.group_id, p.permission.clone()))
            .collect(),
        group_members: HashMap::new(),
    };
    let group_ids: Vec<u32> = rows
        .user_groups
        .iter()
        .chain(&rows.role_groups)
        .map(|(g, _)| *g)
        .collect();
    if !group_ids.is_empty() {
        for gm in repo::groups_monitors::find_by_group_ids(db, &group_ids).await? {
            rows.group_members
                .entry(gm.group_id)
                .or_default()
                .push(gm.monitor_id);
        }
    }
    Ok(scope_from(&rows))
}

/// Extract a bearer token — `Authorization: Bearer <jwt>` header, or the
/// `?token=` query parameter used by HTML media elements.
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

/// Resolve the caller's scope from a token in the given headers/URI.
async fn resolve_from_request(
    state: &AppState,
    headers: &HeaderMap,
    uri: &Uri,
) -> AppResult<MonitorScope> {
    let token = extract_token(headers, uri)
        .ok_or_else(|| AppError::UnauthorizedError("Authentication required".to_string()))?;
    let claims = UserClaims::decode_access(&token)
        .map_err(|_| AppError::UnauthorizedError("Invalid token".to_string()))?
        .claims;
    resolve(state.db(), claims.uid).await
}

/// Axum extractor that resolves the caller's [`MonitorScope`].
///
/// Self-contained — it decodes the access token itself (header or query), so
/// it does not depend on auth-middleware ordering. The scope is resolved live
/// from the database, so monitor-permission changes take effect immediately.
impl FromRequestParts<AppState> for MonitorScope {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // A signed media URL sees one monitor, read-only.
        if let Some(grant) = parts
            .extensions
            .get::<crate::service::media_grant::MediaGrant>()
        {
            return Ok(grant.scope());
        }
        resolve_from_request(state, &parts.headers, &parts.uri).await
    }
}

/// Route-layer guard for endpoints whose monitor id is a *path parameter*
/// (`{monitor_id}` or `{id}`) — streaming and PTZ routes, where the monitor is
/// named in the URL rather than discovered from a query result.
///
/// Apply with `Router::route_layer` so it runs after routing (path params are
/// populated). Out-of-scope monitors yield 404.
pub async fn monitor_path_guard(
    State(state): State<AppState>,
    params: RawPathParams,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let monitor_id = params
        .iter()
        .find(|(key, _)| *key == "monitor_id" || *key == "id")
        .and_then(|(_, value)| value.parse::<u32>().ok());

    if let Some(mid) = monitor_id {
        let scope = match request
            .extensions()
            .get::<crate::service::media_grant::MediaGrant>()
        {
            Some(grant) => grant.scope(),
            None => resolve_from_request(&state, request.headers(), request.uri()).await?,
        };
        let required = match *request.method() {
            Method::GET | Method::HEAD | Method::OPTIONS => Level::View,
            _ => Level::Edit,
        };
        if !scope.allows(mid, required) {
            return Err(AppError::NotFoundError(Resource {
                details: vec![("monitor_id".to_string(), mid.to_string())],
                resource_type: ResourceType::Monitor,
            }));
        }
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> AclRows {
        AclRows {
            group_members: HashMap::from([(10, vec![1, 2]), (20, vec![3])]),
            ..Default::default()
        }
    }

    #[test]
    fn no_rows_anywhere_is_unrestricted() {
        assert!(matches!(scope_from(&rows()), MonitorScope::All));
    }

    #[test]
    fn a_role_alone_restricts_like_the_users_own_rows_would() {
        let mut r = rows();
        r.role_groups = vec![(10, Permission::View)];
        r.role_monitors = vec![(2, Permission::Edit)];
        let s = scope_from(&r);
        assert!(s.allows(1, Level::View) && !s.allows(1, Level::Edit));
        assert!(s.allows(2, Level::Edit));
        assert!(!s.allows(3, Level::View));
    }

    #[test]
    fn the_users_own_grants_override_the_roles() {
        let mut r = rows();
        r.role_groups = vec![(10, Permission::Edit)];
        r.role_monitors = vec![(3, Permission::View)];
        // The user's own group grant lowers 1 and 2, their direct grant
        // denies 2, and an Inherit row leaves the role's 3 alone.
        r.user_groups = vec![(10, Permission::View)];
        r.user_monitors = vec![(2, Permission::None), (3, Permission::Inherit)];
        let s = scope_from(&r);
        assert!(s.allows(1, Level::View) && !s.allows(1, Level::Edit));
        assert!(!s.allows(2, Level::View));
        assert!(s.allows(3, Level::View));
    }
    use crate::entity::{groups_monitors, groups_permissions, monitors_permissions};
    use sea_orm::{DatabaseBackend, MockDatabase};

    fn mon_perm(id: u32, monitor_id: u32, permission: Permission) -> monitors_permissions::Model {
        monitors_permissions::Model {
            id,
            monitor_id,
            user_id: 7,
            permission,
        }
    }

    fn grp_perm(id: u32, group_id: u32, permission: Permission) -> groups_permissions::Model {
        groups_permissions::Model {
            id,
            group_id,
            user_id: 7,
            permission,
        }
    }

    fn grp_mon(id: u32, group_id: u32, monitor_id: u32) -> groups_monitors::Model {
        groups_monitors::Model {
            id,
            group_id,
            monitor_id,
        }
    }

    #[tokio::test]
    async fn no_rows_is_unrestricted() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<monitors_permissions::Model, _, _>(vec![vec![]])
            .append_query_results::<groups_permissions::Model, _, _>(vec![vec![]])
            // The user has no role.
            .append_query_results::<crate::entity::users::Model, _, _>(vec![vec![]])
            .into_connection();
        assert_eq!(resolve(&db, 7).await.unwrap(), MonitorScope::All);
    }

    #[tokio::test]
    async fn direct_grants_form_an_allowlist() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![vec![
                mon_perm(1, 10, Permission::View),
                mon_perm(2, 11, Permission::Edit),
            ]])
            .append_query_results::<groups_permissions::Model, _, _>(vec![vec![]])
            // The user has no role.
            .append_query_results::<crate::entity::users::Model, _, _>(vec![vec![]])
            .into_connection();

        let scope = resolve(&db, 7).await.unwrap();
        assert!(scope.is_restricted());
        // Monitor 10: read but not write.
        assert!(scope.allows(10, Level::View));
        assert!(!scope.allows(10, Level::Edit));
        // Monitor 11: read and write.
        assert!(scope.allows(11, Level::Edit));
        // Monitor 99: no row at all.
        assert!(!scope.allows(99, Level::View));
    }

    #[tokio::test]
    async fn group_membership_is_inherited() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results::<monitors_permissions::Model, _, _>(vec![vec![]])
            .append_query_results(vec![vec![grp_perm(1, 5, Permission::View)]])
            // The user has no role.
            .append_query_results::<crate::entity::users::Model, _, _>(vec![vec![]])
            .append_query_results(vec![vec![grp_mon(1, 5, 20), grp_mon(2, 5, 21)]])
            .into_connection();

        let scope = resolve(&db, 7).await.unwrap();
        assert!(scope.allows(20, Level::View));
        assert!(scope.allows(21, Level::View));
        // Group grant is View only.
        assert!(!scope.allows(20, Level::Edit));
    }

    #[tokio::test]
    async fn direct_none_overrides_a_group_grant() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![vec![mon_perm(1, 20, Permission::None)]])
            .append_query_results(vec![vec![grp_perm(1, 5, Permission::Edit)]])
            // The user has no role.
            .append_query_results::<crate::entity::users::Model, _, _>(vec![vec![]])
            .append_query_results(vec![vec![grp_mon(1, 5, 20), grp_mon(2, 5, 21)]])
            .into_connection();

        let scope = resolve(&db, 7).await.unwrap();
        // Monitor 20 is explicitly denied despite the group granting Edit.
        assert!(!scope.allows(20, Level::View));
        // Monitor 21 keeps the group-granted Edit.
        assert!(scope.allows(21, Level::Edit));
    }

    #[tokio::test]
    async fn direct_inherit_keeps_the_group_level() {
        let db = MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results(vec![vec![mon_perm(1, 20, Permission::Inherit)]])
            .append_query_results(vec![vec![grp_perm(1, 5, Permission::Edit)]])
            // The user has no role.
            .append_query_results::<crate::entity::users::Model, _, _>(vec![vec![]])
            .append_query_results(vec![vec![grp_mon(1, 5, 20)]])
            .into_connection();

        let scope = resolve(&db, 7).await.unwrap();
        assert!(scope.allows(20, Level::Edit));
    }

    #[tokio::test]
    async fn visible_ids_filters_by_required_level() {
        let mut map = HashMap::new();
        map.insert(10, Level::View);
        map.insert(11, Level::Edit);
        let scope = MonitorScope::Restricted(map);

        let mut view = scope.visible_ids(Level::View).unwrap();
        view.sort_unstable();
        assert_eq!(view, vec![10, 11]);

        let edit = scope.visible_ids(Level::Edit).unwrap();
        assert_eq!(edit, vec![11]);

        assert_eq!(MonitorScope::All.visible_ids(Level::View), None);
    }
}
