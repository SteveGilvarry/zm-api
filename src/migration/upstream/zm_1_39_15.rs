//! Mirrors `db/legacy/zm_update-1.39.15.sql`: the HiSilicon Hi3510 control.

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
            "HiSilicon_Hi3510_CGI",
            "HiSilicon Hi3510 CGI",
            "Ffmpeg",
            "HiSilicon_Hi3510_CGI",
            &[
                ("CanReset", 1),
                ("HasPresets", 1),
                ("NumPresets", 10),
                ("CanSetPresets", 1),
                ("CanMove", 1),
                ("CanMoveDiag", 1),
                ("CanMoveCon", 1),
                ("CanPan", 1),
                ("CanTilt", 1),
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
