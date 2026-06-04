//! Database fixture builders for integration tests.
//!
//! Every fixture row is named with [`super::test_db::test_prefix`] so an
//! entire test run's data can be removed with a single prefixed `DELETE`.
#![allow(dead_code)]

use std::future::Future;
use std::pin::Pin;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use zm_api::entity::devices::{
    ActiveModel as DeviceActiveModel, Entity as DeviceEntity, Model as DeviceModel,
};
use zm_api::entity::groups::{
    ActiveModel as GroupActiveModel, Entity as GroupEntity, Model as GroupModel,
};
use zm_api::entity::groups_permissions::{
    ActiveModel as GroupPermActiveModel, Column as GroupPermColumn, Entity as GroupPermEntity,
};
use zm_api::entity::monitors::{ActiveModel as MonitorActiveModel, Model as MonitorModel};
use zm_api::entity::monitors_permissions::{
    ActiveModel as MonitorPermActiveModel, Column as MonitorPermColumn, Entity as MonitorPermEntity,
};
use zm_api::entity::sea_orm_active_enums::{DeviceType, Permission};
use zm_api::entity::users::{ActiveModel as UserActiveModel, Entity as UserEntity};

use super::test_db::{cleanup_by_prefix, get_test_db, test_prefix};

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

/// Insert a throwaway group and return the persisted model.
pub async fn insert_group(db: &DatabaseConnection, label: &str) -> Result<GroupModel, DbErr> {
    GroupActiveModel {
        name: Set(unique_name(label)),
        parent_id: Set(None),
        ..Default::default()
    }
    .insert(db)
    .await
}

/// Delete a single group row by id.
pub async fn delete_group(db: &DatabaseConnection, group_id: u32) -> Result<(), DbErr> {
    GroupEntity::delete_by_id(group_id).exec(db).await?;
    Ok(())
}

/// Grant a user a row-level `Groups_Permissions` permission on a group.
pub async fn grant_group_permission(
    db: &DatabaseConnection,
    group_id: u32,
    user_id: u32,
    permission: Permission,
) -> Result<(), DbErr> {
    GroupPermActiveModel {
        group_id: Set(group_id),
        user_id: Set(user_id),
        permission: Set(permission),
        ..Default::default()
    }
    .insert(db)
    .await?;
    Ok(())
}

/// Delete all `Groups_Permissions` rows for a (test) user.
pub async fn cleanup_group_permissions(db: &DatabaseConnection, user_id: u32) -> Result<(), DbErr> {
    GroupPermEntity::delete_many()
        .filter(GroupPermColumn::UserId.eq(user_id))
        .exec(db)
        .await?;
    Ok(())
}

/// Boxed async cleanup closure run when a [`RowGuard`] is dropped.
type CleanupFn =
    Box<dyn FnOnce(DatabaseConnection) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

/// RAII guard that deletes a fixture row by primary key when it goes out of
/// scope — including when a test panics partway through.
///
/// Integration tests historically deleted their fixture rows at the *end* of
/// the test body, so a failing assertion (a panic) skipped the cleanup and
/// leaked the row into the database. Binding the created id to a `RowGuard`
/// immediately after creation moves that delete into `Drop`, which runs during
/// unwinding too — closing the leak.
///
/// ## Why a dedicated thread + fresh connection
///
/// `Drop` is synchronous, but the delete is async. We cannot reuse the test's
/// `DatabaseConnection`: a SeaORM/`sqlx` pool is bound to the runtime that
/// created it, and `#[tokio::test]` defaults to a single-threaded runtime, so
/// neither `Handle::block_on` (re-entrant) nor `block_in_place` (multi-thread
/// only) is available from within `Drop`. Instead the guard spawns a short
/// std thread, builds its own current-thread runtime there, opens a fresh
/// connection via [`get_test_db`], runs the delete, and joins — fully isolated
/// from the test's runtime and safe under panic.
///
/// Deletes are idempotent (delete-by-id of a missing row is a no-op), so a
/// guard is harmless even when the test already removed the row through the
/// API under test. When a test wants to assert that no second delete happens,
/// call [`RowGuard::disarm`].
#[must_use = "bind the guard to a variable (e.g. `let _guard = ...`) so it lives until the test ends"]
pub struct RowGuard {
    cleanup: Option<CleanupFn>,
    label: String,
}

impl RowGuard {
    /// Build a guard from an arbitrary async cleanup closure.
    ///
    /// Prefer the typed constructors below; this exists for rows with shapes
    /// they don't cover (e.g. extra child rows to remove in one guard).
    pub fn new<F, Fut>(label: impl Into<String>, cleanup: F) -> Self
    where
        F: FnOnce(DatabaseConnection) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        Self {
            cleanup: Some(Box::new(move |db| Box::pin(cleanup(db)))),
            label: label.into(),
        }
    }

    /// Disarm the guard so it performs no cleanup on drop. Use when the test
    /// has already deleted the row through the code under test.
    pub fn disarm(mut self) {
        self.cleanup = None;
    }

    /// Guard a `Monitors` row.
    pub fn monitor(id: u32) -> Self {
        Self::new(format!("Monitors#{id}"), move |db| async move {
            let _ = zm_api::entity::monitors::Entity::delete_by_id(id)
                .exec(&db)
                .await;
        })
    }

    /// Guard a `Groups` row.
    pub fn group(id: u32) -> Self {
        Self::new(format!("Groups#{id}"), move |db| async move {
            let _ = zm_api::entity::groups::Entity::delete_by_id(id)
                .exec(&db)
                .await;
        })
    }

    /// Guard a `Groups_Monitors` row.
    pub fn group_monitor(id: u32) -> Self {
        Self::new(format!("Groups_Monitors#{id}"), move |db| async move {
            let _ = zm_api::entity::groups_monitors::Entity::delete_by_id(id)
                .exec(&db)
                .await;
        })
    }

    /// Guard a `Devices` row.
    pub fn device(id: u32) -> Self {
        Self::new(format!("Devices#{id}"), move |db| async move {
            let _ = DeviceEntity::delete_by_id(id).exec(&db).await;
        })
    }

    /// Guard a `Filters` row.
    pub fn filter(id: u32) -> Self {
        Self::new(format!("Filters#{id}"), move |db| async move {
            let _ = zm_api::entity::filters::Entity::delete_by_id(id)
                .exec(&db)
                .await;
        })
    }

    /// Guard a `Zones` row.
    pub fn zone(id: u32) -> Self {
        Self::new(format!("Zones#{id}"), move |db| async move {
            let _ = zm_api::entity::zones::Entity::delete_by_id(id)
                .exec(&db)
                .await;
        })
    }

    /// Guard a `Users` row.
    pub fn user(id: u32) -> Self {
        Self::new(format!("Users#{id}"), move |db| async move {
            let _ = UserEntity::delete_by_id(id).exec(&db).await;
        })
    }

    /// Guard a `MontageLayouts` row.
    pub fn montage_layout(id: u32) -> Self {
        Self::new(format!("MontageLayouts#{id}"), move |db| async move {
            let _ = zm_api::entity::montage_layouts::Entity::delete_by_id(id)
                .exec(&db)
                .await;
        })
    }

    /// Guard a `Reports` row.
    pub fn report(id: u32) -> Self {
        Self::new(format!("Reports#{id}"), move |db| async move {
            let _ = zm_api::entity::reports::Entity::delete_by_id(id)
                .exec(&db)
                .await;
        })
    }

    /// Guard a `States` row.
    pub fn state(id: u32) -> Self {
        Self::new(format!("States#{id}"), move |db| async move {
            let _ = zm_api::entity::states::Entity::delete_by_id(id)
                .exec(&db)
                .await;
        })
    }

    /// Guard a `MonitorPresets` row.
    pub fn monitor_preset(id: u32) -> Self {
        Self::new(format!("MonitorPresets#{id}"), move |db| async move {
            let _ = zm_api::entity::monitor_presets::Entity::delete_by_id(id)
                .exec(&db)
                .await;
        })
    }

    /// Guard a `ZonePresets` row.
    pub fn zone_preset(id: u32) -> Self {
        Self::new(format!("ZonePresets#{id}"), move |db| async move {
            let _ = zm_api::entity::zone_presets::Entity::delete_by_id(id)
                .exec(&db)
                .await;
        })
    }

    /// Guard a `Controls` row.
    pub fn control(id: u32) -> Self {
        Self::new(format!("Controls#{id}"), move |db| async move {
            let _ = zm_api::entity::controls::Entity::delete_by_id(id)
                .exec(&db)
                .await;
        })
    }

    /// Guard a `ControlPresets` row (composite key: monitor id + preset).
    pub fn control_preset(monitor_id: u32, preset: u32) -> Self {
        Self::new(
            format!("ControlPresets#({monitor_id},{preset})"),
            move |db| async move {
                let _ = zm_api::entity::control_presets::Entity::delete_by_id((monitor_id, preset))
                    .exec(&db)
                    .await;
            },
        )
    }

    /// Guard a `Manufacturers` row.
    pub fn manufacturer(id: u32) -> Self {
        Self::new(format!("Manufacturers#{id}"), move |db| async move {
            let _ = zm_api::entity::manufacturers::Entity::delete_by_id(id)
                .exec(&db)
                .await;
        })
    }

    /// Guard a `Servers` row.
    pub fn server(id: u32) -> Self {
        Self::new(format!("Servers#{id}"), move |db| async move {
            let _ = zm_api::entity::servers::Entity::delete_by_id(id)
                .exec(&db)
                .await;
        })
    }

    /// Guard a `Storage` row.
    pub fn storage(id: u16) -> Self {
        Self::new(format!("Storage#{id}"), move |db| async move {
            let _ = zm_api::entity::storage::Entity::delete_by_id(id)
                .exec(&db)
                .await;
        })
    }

    /// Guard a `Tags` row.
    pub fn tag(id: u64) -> Self {
        Self::new(format!("Tags#{id}"), move |db| async move {
            let _ = zm_api::entity::tags::Entity::delete_by_id(id)
                .exec(&db)
                .await;
        })
    }

    /// Guard a `TriggersX10` row (keyed by monitor id).
    pub fn triggers_x10(monitor_id: u32) -> Self {
        Self::new(format!("TriggersX10#{monitor_id}"), move |db| async move {
            let _ = zm_api::entity::triggers_x10::Entity::delete_by_id(monitor_id)
                .exec(&db)
                .await;
        })
    }
}

impl Drop for RowGuard {
    fn drop(&mut self) {
        let Some(cleanup) = self.cleanup.take() else {
            return;
        };
        let label = std::mem::take(&mut self.label);
        // Run the async delete to completion on a dedicated thread with its own
        // runtime and connection — see the type-level docs for why we cannot
        // reuse the test's runtime/connection here.
        let result = std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("RowGuard[{label}]: failed to build cleanup runtime: {e}");
                    return;
                }
            };
            rt.block_on(async move {
                match get_test_db().await {
                    Ok(db) => cleanup(db).await,
                    Err(e) => eprintln!("RowGuard[{label}]: failed to connect for cleanup: {e}"),
                }
            });
        })
        .join();
        if result.is_err() {
            eprintln!("RowGuard cleanup thread panicked");
        }
    }
}
