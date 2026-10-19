//! `SeaORM` Entity for ZoneMinder 1.39.30's `MonitorActions`: what a monitor
//! does to another monitor's device on a trigger. Curated by hand to match
//! `zm_create.sql.in`; the two enum columns are read as strings and typed in
//! the DTOs.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "MonitorActions")]
pub struct Model {
    #[sea_orm(column_name = "Id", primary_key)]
    pub id: u32,
    #[sea_orm(column_name = "MonitorId")]
    pub monitor_id: u32,
    #[sea_orm(column_name = "TriggerOn")]
    pub trigger_on: String,
    #[sea_orm(column_name = "ActionType")]
    pub action_type: String,
    #[sea_orm(column_name = "TargetMonitorId")]
    pub target_monitor_id: u32,
    #[sea_orm(column_name = "AudioFile")]
    pub audio_file: Option<u32>,
    #[sea_orm(column_name = "Label")]
    pub label: String,
    #[sea_orm(column_name = "Enabled")]
    pub enabled: u8,
    #[sea_orm(column_name = "Sequence")]
    pub sequence: u16,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
