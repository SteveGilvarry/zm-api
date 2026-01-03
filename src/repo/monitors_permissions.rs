use crate::dto::request::monitors_permissions::{
    CreateMonitorPermissionRequest, UpdateMonitorPermissionRequest,
};
use crate::entity::monitors_permissions::{
    ActiveModel, Column, Entity as MonitorsPermissions, Model as MonitorPermissionModel,
};
use crate::entity::sea_orm_active_enums::Permission;
use crate::error::{AppError, AppResult};
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<MonitorPermissionModel>> {
    Ok(MonitorsPermissions::find().all(db).await?)
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
