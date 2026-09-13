//! Mirrors `db/legacy/zm_update-1.39.16.sql`: two LTS HikVision controls and
//! `Menu_Items.Link`.

use sea_orm_migration::prelude::*;

use super::controls::add_control;
use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        for name in ["LTS CMIP1342WE-28MDA", "LTS CMIP3CD42WI-28AISP"] {
            add_control(
                m,
                name,
                "Ffmpeg",
                "HikVision",
                &[("CanReset", 0), ("CanReboot", 1), ("CanLight", 1)],
            )
            .await?;
        }
        add_column_if_missing(
            m,
            "Menu_Items",
            "Link",
            ColumnDef::new(c("Link")).string_len(255).to_owned(),
        )
        .await
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}
