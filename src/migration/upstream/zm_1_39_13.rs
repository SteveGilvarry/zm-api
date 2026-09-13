//! Mirrors `db/legacy/zm_update-1.39.13.sql`: `Controls.CanLight`.

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        add_column_if_missing(
            m,
            "Controls",
            "CanLight",
            ColumnDef::new(c("CanLight"))
                .tiny_unsigned()
                .not_null()
                .default("0")
                .to_owned(),
        )
        .await?;
        exec_stmt(
            m,
            Query::update()
                .table(t("Controls"))
                .value(c("CanLight"), 1)
                .and_where(Expr::col(c("Name")).eq("Amcrest ADC2W RPC")),
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
