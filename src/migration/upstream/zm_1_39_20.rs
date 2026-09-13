//! Mirrors `db/legacy/zm_update-1.39.20.sql`: the Amcrest IP5M-1190EW control.

use sea_orm_migration::prelude::*;

use super::controls::add_control;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_control(
            m,
            "Amcrest IP5M-1190EW HTTP",
            "Ffmpeg",
            "Amcrest_HTTP",
            &[
                ("CanReset", 1),
                ("CanReboot", 1),
                ("HasPresets", 1),
                ("NumPresets", 25),
                ("HasHomePreset", 1),
                ("CanSetPresets", 1),
                ("CanMove", 1),
                ("CanMoveCon", 1),
                ("CanPan", 1),
                ("HasPanSpeed", 1),
                ("MinPanSpeed", 1),
                ("MaxPanSpeed", 4),
                ("CanTilt", 1),
                ("HasTiltSpeed", 1),
                ("MinTiltSpeed", 1),
                ("MaxTiltSpeed", 4),
            ],
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
