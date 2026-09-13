//! Mirrors `db/legacy/zm_update-1.39.5.sql`: `Monitors.WhatDisplay`.

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const VALUES: [&str; 3] = [
    "OnlyVideo",
    "OnlyAudioVisualization",
    "VideoAudioVisualization",
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        ensure_enum_type(m, "monitors_what_display", &VALUES).await?;
        let def = enum_col("WhatDisplay", "monitors_what_display", &VALUES)
            .not_null()
            .default("OnlyVideo")
            .to_owned();
        if column_exists(m, "Monitors", "WhatDisplay").await? {
            modify_column(m, "Monitors", def).await
        } else {
            add_column_if_missing(m, "Monitors", "WhatDisplay", def).await
        }
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}
