use crate::dto::request::monitors_permissions::{
    CreateMonitorPermissionRequest, UpdateMonitorPermissionRequest,
};
use crate::dto::PaginationParams;
use crate::entity::monitors_permissions::{
    ActiveModel, Column, Entity as MonitorsPermissions, Model as MonitorPermissionModel,
};
use crate::entity::sea_orm_active_enums::Permission;
use crate::error::{AppError, AppResult};
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<MonitorPermissionModel>> {
    Ok(MonitorsPermissions::find().all(db).await?)
}

pub async fn find_paginated(
    db: &DatabaseConnection,
    params: &PaginationParams,
) -> AppResult<(Vec<MonitorPermissionModel>, u64)> {
    let paginator = MonitorsPermissions::find().paginate(db, params.page_size());
    let total = paginator.num_items().await?;
    let items = paginator
        .fetch_page(params.page().saturating_sub(1))
        .await?;
    Ok((items, total))
}

pub async fn find_by_id(
    db: &DatabaseConnection,
    id: u32,
) -> AppResult<Option<MonitorPermissionModel>> {
    Ok(MonitorsPermissions::find_by_id(id).one(db).await?)
}

pub async fn find_by_monitor_id(
    db: &DatabaseConnection,
    monitor_id: u32,
) -> AppResult<Vec<MonitorPermissionModel>> {
    Ok(MonitorsPermissions::find()
        .filter(Column::MonitorId.eq(monitor_id))
        .all(db)
        .await?)
}

pub async fn find_by_user_id(
    db: &DatabaseConnection,
    user_id: u32,
) -> AppResult<Vec<MonitorPermissionModel>> {
    Ok(MonitorsPermissions::find()
        .filter(Column::UserId.eq(user_id))
        .all(db)
        .await?)
}

/// Which permission table a [`UserGrant`] row came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantKind {
    /// `Monitors_Permissions`: `target_id` is a monitor id.
    Monitor = 0,
    /// `Groups_Permissions`: `target_id` is a group id.
    Group = 1,
    /// `Role_Monitors_Permissions` of the user's role: `target_id` is a monitor id.
    RoleMonitor = 2,
    /// `Role_Groups_Permissions` of the user's role: `target_id` is a group id.
    RoleGroup = 3,
}

/// One row of a user's permission grants, direct or through their role.
#[derive(Debug, Clone, PartialEq, Eq, FromQueryResult)]
pub struct UserGrant {
    /// A [`GrantKind`] discriminant.
    pub kind: i32,
    pub target_id: u32,
    pub permission: Permission,
}

impl UserGrant {
    pub fn kind(&self) -> GrantKind {
        match self.kind {
            1 => GrantKind::Group,
            2 => GrantKind::RoleMonitor,
            3 => GrantKind::RoleGroup,
            _ => GrantKind::Monitor,
        }
    }
}

/// Every monitor and group grant that applies to a user, in one round trip:
/// their own `Monitors_Permissions` and `Groups_Permissions` rows plus those of
/// the role named by `Users.RoleId`. Row-level ACL resolution runs on every
/// monitor-scoped request, and most users have no rows at all.
pub async fn find_user_grants<C: ConnectionTrait>(
    db: &C,
    user_id: u32,
) -> AppResult<Vec<UserGrant>> {
    use crate::entity::groups_permissions;
    use sea_orm::sea_query::{Alias, Expr, Query, SelectStatement, UnionType};

    // Role grant tables have no generated entity; name them directly.
    fn role_grants(table: &str, target: &str, kind: GrantKind, user_id: u32) -> SelectStatement {
        let t = Alias::new(table);
        let users = Alias::new("Users");
        Query::select()
            .expr_as(Expr::val(kind as i32), "kind")
            .expr_as(Expr::col((t.clone(), Alias::new(target))), "target_id")
            .expr_as(
                Expr::col((t.clone(), Alias::new("Permission"))),
                "permission",
            )
            .from(t.clone())
            .inner_join(
                users.clone(),
                Expr::col((users.clone(), Alias::new("RoleId")))
                    .equals((t.clone(), Alias::new("RoleId"))),
            )
            .and_where(Expr::col((users, Alias::new("Id"))).eq(user_id))
            .to_owned()
    }

    let mut query = Query::select()
        .expr_as(Expr::val(GrantKind::Monitor as i32), "kind")
        .expr_as(Expr::col(Column::MonitorId), "target_id")
        .expr_as(Expr::col(Column::Permission), "permission")
        .from(MonitorsPermissions)
        .and_where(Column::UserId.eq(user_id))
        .to_owned();
    query
        .union(
            UnionType::All,
            Query::select()
                .expr_as(Expr::val(GrantKind::Group as i32), "kind")
                .expr_as(Expr::col(groups_permissions::Column::GroupId), "target_id")
                .expr_as(
                    Expr::col(groups_permissions::Column::Permission),
                    "permission",
                )
                .from(groups_permissions::Entity)
                .and_where(groups_permissions::Column::UserId.eq(user_id))
                .to_owned(),
        )
        .union(
            UnionType::All,
            role_grants(
                "Role_Monitors_Permissions",
                "MonitorId",
                GrantKind::RoleMonitor,
                user_id,
            ),
        )
        .union(
            UnionType::All,
            role_grants(
                "Role_Groups_Permissions",
                "GroupId",
                GrantKind::RoleGroup,
                user_id,
            ),
        );
    let stmt = db.get_database_backend().build(&query);
    Ok(UserGrant::find_by_statement(stmt).all(db).await?)
}

pub async fn create(
    db: &DatabaseConnection,
    req: &CreateMonitorPermissionRequest,
) -> AppResult<MonitorPermissionModel> {
    let permission = match req.permission.as_str() {
        "Inherit" => Permission::Inherit,
        "None" => Permission::None,
        "View" => Permission::View,
        "Edit" => Permission::Edit,
        _ => {
            return Err(AppError::BadRequestError(format!(
                "Invalid permission: {}",
                req.permission
            )))
        }
    };

    let am = ActiveModel {
        id: Default::default(),
        monitor_id: Set(req.monitor_id),
        user_id: Set(req.user_id),
        permission: Set(permission),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(
    db: &DatabaseConnection,
    id: u32,
    req: &UpdateMonitorPermissionRequest,
) -> AppResult<Option<MonitorPermissionModel>> {
    let Some(model) = find_by_id(db, id).await? else {
        return Ok(None);
    };
    let mut am: ActiveModel = model.into();

    if let Some(perm_str) = &req.permission {
        let permission = match perm_str.as_str() {
            "Inherit" => Permission::Inherit,
            "None" => Permission::None,
            "View" => Permission::View,
            "Edit" => Permission::Edit,
            _ => {
                return Err(AppError::BadRequestError(format!(
                    "Invalid permission: {}",
                    perm_str
                )))
            }
        };
        am.permission = Set(permission);
    }

    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    let res = MonitorsPermissions::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
