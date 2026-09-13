//! Mirrors `db/legacy/zm_update-1.39.31.sql`: audio-detection columns on `Monitors`.

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_column_if_missing(
            m,
            "Monitors",
            "AudioDetection",
            ColumnDef::new(c("AudioDetection"))
                .tiny_unsigned()
                .not_null()
                .default("0")
                .to_owned(),
        )
        .await?;
        add_column_if_missing(
            m,
            "Monitors",
            "AudioThreshold",
            ColumnDef::new(c("AudioThreshold"))
                .tiny_unsigned()
                .not_null()
                .default("0")
                .to_owned(),
        )
        .await?;
        add_column_if_missing(
            m,
            "Monitors",
            "AudioAlarmScore",
            ColumnDef::new(c("AudioAlarmScore"))
                .small_unsigned()
                .not_null()
                .default("9")
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
