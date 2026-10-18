//! Mirrors `db/legacy/zm_update-1.39.34.sql`: Controls still pointing at the
//! removed `ZoneMinder::Control::onvif` module move to `ONVIF`.

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        exec_stmt(
            m,
            Query::update()
                .table(t("Controls"))
                .value(c("Protocol"), "ONVIF")
                .and_where(Expr::col(c("Protocol")).eq("onvif")),
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
