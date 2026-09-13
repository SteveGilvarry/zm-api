//! Mirrors `db/legacy/zm_update-1.39.33.sql`: `MonitorActions.TriggerOn`
//! gains `AlarmEnd`.

use sea_orm_migration::prelude::*;

use super::helpers::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const TRIGGER_ON: [&str; 5] = ["EventStart", "EventEnd", "Alarm", "AlarmEnd", "Manual"];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        widen_enum(
            m,
            "MonitorActions",
            "TriggerOn",
            "monitor_actions_trigger_on",
            &TRIGGER_ON,
            enum_col("TriggerOn", "monitor_actions_trigger_on", &TRIGGER_ON)
                .not_null()
                .default("EventStart")
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
