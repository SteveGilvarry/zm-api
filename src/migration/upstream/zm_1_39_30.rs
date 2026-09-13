//! Mirrors `db/legacy/zm_update-1.39.30.sql`: `Monitors.DeviceClass` and the
//! `MonitorActions` table.

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

pub(super) const TRIGGER_ON_1_39_30: [&str; 4] = ["EventStart", "EventEnd", "Alarm", "Manual"];
const ACTION_TYPES: [&str; 6] = [
    "LightOn",
    "LightOff",
    "IndicatorLightOn",
    "IndicatorLightOff",
    "AudioPlay",
    "AudioStop",
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        ensure_enum_type(m, "monitors_device_class", &["Camera", "Speaker"]).await?;
        add_column_if_missing(
            m,
            "Monitors",
            "DeviceClass",
            enum_col(
                "DeviceClass",
                "monitors_device_class",
                &["Camera", "Speaker"],
            )
            .not_null()
            .default("Camera")
            .to_owned(),
        )
        .await?;

        if table_exists(m, "MonitorActions").await? {
            return Ok(());
        }
        ensure_enum_type(m, "monitor_actions_trigger_on", &TRIGGER_ON_1_39_30).await?;
        ensure_enum_type(m, "monitor_actions_action_type", &ACTION_TYPES).await?;
        create_table(
            m,
            Table::create()
                .table(t("MonitorActions"))
                .col(autoinc_pk(m, "Id", false))
                .col(ColumnDef::new(c("MonitorId")).unsigned().not_null())
                .col(
                    enum_col(
                        "TriggerOn",
                        "monitor_actions_trigger_on",
                        &TRIGGER_ON_1_39_30,
                    )
                    .not_null()
                    .default("EventStart"),
                )
                .col(
                    enum_col("ActionType", "monitor_actions_action_type", &ACTION_TYPES)
                        .not_null()
                        .default("AudioPlay"),
                )
                .col(ColumnDef::new(c("TargetMonitorId")).unsigned().not_null())
                .col(ColumnDef::new(c("AudioFile")).unsigned())
                .col(
                    ColumnDef::new(c("Label"))
                        .string_len(64)
                        .not_null()
                        .default(""),
                )
                .col(
                    ColumnDef::new(c("Enabled"))
                        .tiny_unsigned()
                        .not_null()
                        .default("1"),
                )
                .col(
                    ColumnDef::new(c("Sequence"))
                        .small_unsigned()
                        .not_null()
                        .default("0"),
                )
                .to_owned(),
            "MonitorActions",
            &[
                ("MonitorId_TriggerOn", &["MonitorId", "TriggerOn"], false),
                ("TargetMonitorId", &["TargetMonitorId"], false),
            ],
        )
        .await
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration(
            "upstream ZoneMinder schema updates are not reversible".into(),
        ))
    }
}
