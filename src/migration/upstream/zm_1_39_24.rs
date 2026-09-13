//! Mirrors `db/legacy/zm_update-1.39.24.sql`: `Monitors.FrameSkip` is dropped
//! and the fixed Foscam HD control is added.

use sea_orm_migration::prelude::*;

use super::controls::add_control;
use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_column_if_present(m, "Monitors", "FrameSkip").await?;
        add_control(
            m,
            "Foscam HD (fixed)",
            "Ffmpeg",
            "FoscamHD",
            &[("CanReset", 0), ("CanReboot", 1)],
        )
        .await?;
        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}
