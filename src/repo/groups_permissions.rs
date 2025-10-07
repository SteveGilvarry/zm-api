use sea_orm::*;
use crate::entity::groups_permissions::{Entity as GroupsPermissions, Model as GroupPermissionModel, ActiveModel, Column};
use crate::entity::sea_orm_active_enums::Permission;
use crate::error::{AppResult, AppError};
use crate::dto::request::groups_permissions::{CreateGroupPermissionRequest, UpdateGroupPermissionRequest};

pub async fn find_all(db: &DatabaseConnection) -> AppResult<Vec<GroupPermissionModel>> {
    Ok(GroupsPermissions::find().all(db).await?)
}

pub async fn find_by_id(db: &DatabaseConnection, id: u32) -> AppResult<Option<GroupPermissionModel>> {
    Ok(GroupsPermissions::find_by_id(id).one(db).await?)
}

pub async fn find_by_group_id(db: &DatabaseConnection, group_id: u32) -> AppResult<Vec<GroupPermissionModel>> {
    Ok(GroupsPermissions::find()
        .filter(Column::GroupId.eq(group_id))
        .all(db)
        .await?)
}

pub async fn find_by_user_id(db: &DatabaseConnection, user_id: u32) -> AppResult<Vec<GroupPermissionModel>> {
    Ok(GroupsPermissions::find()
        .filter(Column::UserId.eq(user_id))
        .all(db)
        .await?)
}

pub async fn create(db: &DatabaseConnection, req: &CreateGroupPermissionRequest) -> AppResult<GroupPermissionModel> {
    let permission = match req.permission.as_str() {
        "Inherit" => Permission::Inherit,
        "None" => Permission::None,
        "View" => Permission::View,
        "Edit" => Permission::Edit,
        _ => return Err(AppError::BadRequestError(format!("Invalid permission: {}", req.permission))),
    };
    
    let am = ActiveModel {
        id: Default::default(),
        group_id: Set(req.group_id),
        user_id: Set(req.user_id),
        permission: Set(permission),
    };
    Ok(am.insert(db).await?)
}

pub async fn update(db: &DatabaseConnection, id: u32, req: &UpdateGroupPermissionRequest) -> AppResult<Option<GroupPermissionModel>> {
    let Some(model) = find_by_id(db, id).await? else { return Ok(None) };
    let mut am: ActiveModel = model.into();
    
    if let Some(perm_str) = &req.permission {
        let permission = match perm_str.as_str() {
            "Inherit" => Permission::Inherit,
            "None" => Permission::None,
            "View" => Permission::View,
            "Edit" => Permission::Edit,
            _ => return Err(AppError::BadRequestError(format!("Invalid permission: {}", perm_str))),
        };
        am.permission = Set(permission);
    }
    
    let updated = am.update(db).await?;
    Ok(Some(updated))
}

pub async fn delete_by_id(db: &DatabaseConnection, id: u32) -> AppResult<bool> {
    let res = GroupsPermissions::delete_by_id(id).exec(db).await?;
    Ok(res.rows_affected > 0)
}
