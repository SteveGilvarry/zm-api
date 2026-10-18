//! Mirrors `db/legacy/zm_update-1.39.35.sql`: `Frames.AudioLevel`, the peak
//! audio level since the previous Frames row on the 0-100 scale
//! `Monitors.AudioThreshold` uses.

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_column_if_missing(
            m,
            "Frames",
            "AudioLevel",
            ColumnDef::new(c("AudioLevel"))
                .tiny_unsigned()
                .not_null()
                .default(0)
                .to_owned(),
        )
        .await
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}
