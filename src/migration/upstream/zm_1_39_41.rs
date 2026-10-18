//! Mirrors `db/legacy/zm_update-1.39.41.sql`: a Controls row for
//! `ZoneMinder::Control::Reolink_HTTP` (skipped when one with that protocol
//! exists), and `ZM_OPT_USE_AUTH` defaulting to on.

use sea_orm_migration::prelude::*;

use super::controls::insert_control;
use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        insert_control(
            m,
            "Protocol",
            "Reolink_HTTP",
            "Reolink HTTP API",
            None,
            "Reolink_HTTP",
            &[
                ("CanReboot", 1),
                ("CanZoom", 1),
                ("CanZoomCon", 1),
                ("HasZoomSpeed", 1),
                ("MinZoomSpeed", 1),
                ("MaxZoomSpeed", 64),
                ("CanFocus", 1),
                ("CanAutoFocus", 1),
                ("CanFocusCon", 1),
                ("HasPresets", 1),
                ("NumPresets", 64),
                ("HasHomePreset", 1),
                ("CanSetPresets", 1),
                ("CanMove", 1),
                ("CanMoveDiag", 1),
                ("CanMoveCon", 1),
                ("CanPan", 1),
                ("HasPanSpeed", 1),
                ("MinPanSpeed", 1),
                ("MaxPanSpeed", 64),
                ("CanTilt", 1),
                ("HasTiltSpeed", 1),
                ("MinTiltSpeed", 1),
                ("MaxTiltSpeed", 64),
            ],
        )
        .await?;
        exec_stmt(
            m,
            Query::update()
                .table(t("Config"))
                .value(c("DefaultValue"), "1")
                .and_where(Expr::col(c("Name")).eq("ZM_OPT_USE_AUTH")),
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
