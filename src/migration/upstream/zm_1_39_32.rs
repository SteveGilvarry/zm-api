//! Mirrors `db/legacy/zm_update-1.39.32.sql`: the AMLINK AL5M light control.

use sea_orm_migration::prelude::*;

use super::controls::add_control_keyed;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_control_keyed(
            m,
            "Protocol",
            "AMLink",
            "AMLINK AL5M (light)",
            "Ffmpeg",
            "AMLink",
            &[("CanReboot", 1), ("CanLight", 1)],
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
