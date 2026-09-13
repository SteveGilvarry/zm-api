//! Mirrors `db/legacy/zm_update-1.39.25.sql`: nullability and defaults on
//! several `Monitors` columns and `ZonePresets.Units`.

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
                .table(t("Monitors"))
                .value(c("Janus_Profile_Override"), "")
                .and_where(Expr::col(c("Janus_Profile_Override")).is_null()),
        )
        .await?;
        modify_column(
            m,
            "Monitors",
            ColumnDef::new(c("Janus_Profile_Override"))
                .string_len(30)
                .not_null()
                .default("")
                .to_owned(),
        )
        .await?;
        exec_stmt(
            m,
            Query::update()
                .table(t("Monitors"))
                .value(c("Janus_RTSP_Session_Timeout"), 0)
                .and_where(Expr::col(c("Janus_RTSP_Session_Timeout")).is_null()),
        )
        .await?;
        modify_column(
            m,
            "Monitors",
            ColumnDef::new(c("Janus_RTSP_Session_Timeout"))
                .integer()
                .not_null()
                .default("0")
                .to_owned(),
        )
        .await?;
        modify_column(
            m,
            "Monitors",
            ColumnDef::new(c("MQTT_Subscriptions"))
                .string_len(255)
                .default("")
                .to_owned(),
        )
        .await?;
        modify_column(
            m,
            "Monitors",
            ColumnDef::new(c("OutputCodecName"))
                .string_len(32)
                .not_null()
                .default("auto")
                .to_owned(),
        )
        .await?;
        let containers = ["auto", "mp4", "mkv", "webm"];
        widen_enum(
            m,
            "Monitors",
            "OutputContainer",
            "monitors_output_container",
            &containers,
            enum_col("OutputContainer", "monitors_output_container", &containers),
        )
        .await?;
        let units = ["Pixels", "Percent"];
        widen_enum(
            m,
            "ZonePresets",
            "Units",
            "zone_presets_units",
            &units,
            enum_col("Units", "zone_presets_units", &units)
                .not_null()
                .default("Percent")
                .to_owned(),
        )
        .await
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}
