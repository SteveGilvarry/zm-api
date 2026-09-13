//! Mirrors `db/legacy/zm_update-1.39.22.sql`: the two composite `Events`
//! indexes replace the MonitorId-only and EndDateTime/DiskSpace keys.

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_index_if_missing(
            m,
            "Events",
            "Events_MonitorId_StartDateTime_idx",
            &["MonitorId", "StartDateTime"],
            false,
        )
        .await?;
        add_index_if_missing(
            m,
            "Events",
            "Events_EndDateTime_MonitorId_idx",
            &["EndDateTime", "MonitorId"],
            false,
        )
        .await?;
        for old in [
            "Events_MonitorId_idx",
            "Events_EndDateTime_DiskSpace",
            "Events_EndTime_DiskSpace",
        ] {
            drop_index_if_present(m, "Events", old).await?;
        }
        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}
