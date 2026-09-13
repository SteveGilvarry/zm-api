//! Mirrors `db/legacy/zm_update-1.39.14.sql`: `Controls.CanIndicatorLight`
//! and the Amcrest ASH42-B control.

use sea_orm_migration::prelude::*;

use super::controls::add_control;
use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_column_if_missing(
            m,
            "Controls",
            "CanIndicatorLight",
            ColumnDef::new(c("CanIndicatorLight"))
                .tiny_unsigned()
                .not_null()
                .default("0")
                .to_owned(),
        )
        .await?;
        add_control(
            m,
            "Amcrest ASH42-B RPC",
            "Ffmpeg",
            "Dahua_RPC",
            &[("CanReset", 1), ("CanReboot", 1), ("CanIndicatorLight", 1)],
        )
        .await?;
        exec_stmt(
            m,
            Query::update()
                .table(t("Controls"))
                .value(c("CanIndicatorLight"), 1)
                .and_where(
                    Expr::col(c("Name")).is_in(["Amcrest ASH21-B RPC", "Amcrest ADC2W RPC"]),
                ),
        )
        .await?;
        exec_stmt(
            m,
            Query::update()
                .table(t("Controls"))
                .value(c("CanLight"), 1)
                .value(c("CanIndicatorLight"), 1)
                .and_where(Expr::col(c("Name")).eq("Dahua/Amcrest RPC")),
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
