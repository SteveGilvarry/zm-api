//! Mirrors `db/legacy/zm_update-1.39.40.sql`: the presentation settings move
//! from the System tab to Display (category key `web`).

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const MOVED: [&str; 9] = [
    "ZM_SKIN_DEFAULT",
    "ZM_CSS_DEFAULT",
    "ZM_BANDWIDTH_DEFAULT",
    "ZM_LANG_DEFAULT",
    "ZM_LOCALE_DEFAULT",
    "ZM_DATE_FORMAT_PATTERN",
    "ZM_TIME_FORMAT_PATTERN",
    "ZM_DATETIME_FORMAT_PATTERN",
    "ZM_FEATURES_SNAPSHOTS",
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        exec_stmt(
            m,
            Query::update()
                .table(t("Config"))
                .value(c("Category"), "web")
                .and_where(Expr::col(c("Name")).is_in(MOVED)),
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
