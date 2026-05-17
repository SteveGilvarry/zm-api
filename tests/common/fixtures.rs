//! Database fixture builders for integration tests.
//!
//! Every fixture row is named with [`super::test_db::test_prefix`] so an
//! entire test run's data can be removed with a single prefixed `DELETE`.
#![allow(dead_code)]

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use zm_api::entity::devices::{
    ActiveModel as DeviceActiveModel, Entity as DeviceEntity, Model as DeviceModel,
};
use zm_api::entity::monitors::{ActiveModel as MonitorActiveModel, Model as MonitorModel};
use zm_api::entity::monitors_permissions::{
    ActiveModel as MonitorPermActiveModel, Column as MonitorPermColumn, Entity as MonitorPermEntity,
};
use zm_api::entity::sea_orm_active_enums::{DeviceType, Permission};
use zm_api::entity::users::{ActiveModel as UserActiveModel, Entity as UserEntity};

use super::test_db::{cleanup_by_prefix, test_prefix};

/// A name unique to this test run, so fixture rows can be bulk-cleaned.
pub fn unique_name(label: &str) -> String {
    format!("{}{}", test_prefix(), label)
}

/// Insert a throwaway X10 device and return the persisted model.
pub async fn insert_device(db: &DatabaseConnection, label: &str) -> Result<DeviceModel, DbErr> {
    DeviceActiveModel {
        name: Set(unique_name(label)),
        r#type: Set(DeviceType::X10),
        key_string: Set("A1".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await
}

/// Delete every device row created by this test run.
pub async fn cleanup_devices(db: &DatabaseConnection) -> Result<(), DbErr> {
    cleanup_by_prefix(db, "Devices", "Name", &test_prefix()).await?;
    Ok(())
}

/// Delete a single device row by id.
///
/// Preferred over [`cleanup_devices`] when tests run concurrently — a
/// prefix-wide delete would also remove devices owned by a sibling test
/// (and can race a sibling's list query between its count and fetch).
pub async fn delete_device(db: &DatabaseConnection, device_id: u32) -> Result<(), DbErr> {
    DeviceEntity::delete_by_id(device_id).exec(db).await?;
    Ok(())
}

/// Insert a throwaway monitor and return the persisted model.
pub async fn insert_monitor(db: &DatabaseConnection, label: &str) -> Result<MonitorModel, DbErr> {
    MonitorActiveModel {
        name: Set(unique_name(label)),
        ..Default::default()
    }
    .insert(db)
    .await
}

/// Insert a `Users` row with an explicit id.
///
/// Row-level ACL tests reference a fixed user id; the `Monitors_Permissions`
/// foreign key requires that user to exist first.
pub async fn insert_user_with_id(
    db: &DatabaseConnection,
    user_id: u32,
    label: &str,
) -> Result<(), DbErr> {
    // Idempotent: clear any row left behind by a previously failed run.
    UserEntity::delete_by_id(user_id).exec(db).await?;
    UserActiveModel {
        id: Set(user_id),
        username: Set(unique_name(label)),
        password: Set(String::new()),
        name: Set(unique_name(label)),
        ..Default::default()
    }
    .insert(db)
    .await?;
    Ok(())
}

/// Delete a `Users` row by id (ACL test cleanup).
pub async fn cleanup_user(db: &DatabaseConnection, user_id: u32) -> Result<(), DbErr> {
    UserEntity::delete_by_id(user_id).exec(db).await?;
    Ok(())
}

/// Grant a user a row-level `Monitors_Permissions` permission on a monitor.
pub async fn grant_monitor_permission(
    db: &DatabaseConnection,
    monitor_id: u32,
    user_id: u32,
    permission: Permission,
) -> Result<(), DbErr> {
    MonitorPermActiveModel {
        monitor_id: Set(monitor_id),
        user_id: Set(user_id),
        permission: Set(permission),
        ..Default::default()
    }
    .insert(db)
    .await?;
    Ok(())
}

/// Delete every monitor row created by this test run.
pub async fn cleanup_monitors(db: &DatabaseConnection) -> Result<(), DbErr> {
    cleanup_by_prefix(db, "Monitors", "Name", &test_prefix()).await?;
    Ok(())
}

/// Delete a single monitor row by id.
///
/// Preferred over [`cleanup_monitors`] when tests run concurrently — a
/// prefix-wide delete would also remove monitors owned by a sibling test.
pub async fn delete_monitor(db: &DatabaseConnection, monitor_id: u32) -> Result<(), DbErr> {
    use zm_api::entity::monitors::Entity as MonitorEntity;
    MonitorEntity::delete_by_id(monitor_id).exec(db).await?;
    Ok(())
}

/// Delete all `Monitors_Permissions` rows for a (test) user.
pub async fn cleanup_monitor_permissions(
    db: &DatabaseConnection,
    user_id: u32,
) -> Result<(), DbErr> {
    MonitorPermEntity::delete_many()
        .filter(MonitorPermColumn::UserId.eq(user_id))
        .exec(db)
        .await?;
    Ok(())
}
