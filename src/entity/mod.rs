use sea_orm::{DatabaseTransaction, TransactionTrait};
use test_context::AsyncTestContext;
use tracing::info;

use crate::{
    client::database::{DatabaseClient, DatabaseClientExt},
    constant::CONFIG,
    error::ResourceType,
};
pub mod app_entity_impl;
pub mod config;
pub mod control_presets;
pub mod controls;
pub mod devices;
pub mod event_data;
pub mod event_summaries;
pub mod events;
pub mod events_archived;
pub mod events_day;
pub mod events_hour;
pub mod events_month;
pub mod events_tags;
pub mod events_week;
pub mod filters;
pub mod frames;
pub mod groups;
pub mod groups_monitors;
pub mod groups_permissions;
pub mod logs;
pub mod manufacturers;
pub mod models;
pub mod monitor_presets;
pub mod monitor_status;
pub mod monitors;
pub mod monitors_permissions;
pub mod montage_layouts;
pub mod object_types;
pub mod prelude;
pub mod reports;
pub mod sea_orm_active_enums;
pub mod server_stats;
pub mod servers;
pub mod sessions;
pub mod snapshots;
pub mod snapshots_events;
pub mod states;
pub mod stats;
pub mod storage;
pub mod tags;
pub mod triggers_x10;
pub mod user_preferences;
pub mod users;
pub mod zone_presets;
pub mod zones;

pub trait AppEntity {
    const RESOURCE: ResourceType;
}

pub struct TransactionTestContext {
    pub tx: DatabaseTransaction,
}

impl AsyncTestContext for TransactionTestContext {
    async fn setup() -> Self {
        info!("Setup database for the test.");
        let db = DatabaseClient::build_from_config(&CONFIG).await.unwrap();
        Self {
            tx: db.begin().await.unwrap(),
        }
    }

    async fn teardown(self) {
        self.tx.rollback().await.unwrap();
    }
}

impl std::ops::Deref for TransactionTestContext {
    type Target = DatabaseTransaction;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}
