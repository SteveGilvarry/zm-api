//! Mirrors `db/legacy/zm_update-1.39.4.sql`: `ZM_OPT_USE_REMEMBER_ME` becomes
//! a None/Yes/No option; a former `1` maps to `No`, anything else to `None`.

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const HELP: &str = "\n      Controls whether a \"Remember Me\" checkbox appears on the login page.\n      None: No checkbox is shown. Sessions always persist for ZM_COOKIE_LIFETIME.\n      Yes: Checkbox is shown and checked by default. Users may uncheck it for a session-only cookie.\n      No: Checkbox is shown and unchecked by default. Users may check it to persist the session.\n      ";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let name = "ZM_OPT_USE_REMEMBER_ME";
        // Order matters: rewrite the non-'1' values first, then '1'.
        exec_stmt(
            m,
            Query::update()
                .table(t("Config"))
                .value(c("Value"), "None")
                .and_where(Expr::col(c("Name")).eq(name))
                .and_where(Expr::col(c("Value")).ne("1")),
        )
        .await?;
        exec_stmt(
            m,
            Query::update()
                .table(t("Config"))
                .value(c("Value"), "No")
                .and_where(Expr::col(c("Name")).eq(name))
                .and_where(Expr::col(c("Value")).eq("1")),
        )
        .await?;
        exec_stmt(
            m,
            Query::update()
                .table(t("Config"))
                .value(c("Type"), "string")
                .value(c("DefaultValue"), "None")
                .value(c("Hint"), "None|Yes|No")
                .value(c("Pattern"), "(?^i:^([YyNn]))")
                .value(c("Format"), " $1 ")
                .value(
                    c("Prompt"),
                    "Show a \"Remember Me\" option on the login page",
                )
                .value(c("Help"), HELP)
                .and_where(Expr::col(c("Name")).eq(name)),
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
