//! `SeaORM` Entity for ZoneMinder 1.39's `User_Roles`: a named permission set
//! a user can hold through `Users.RoleId`. Curated by hand to match
//! `zm_create.sql.in`.

use super::sea_orm_active_enums::{
    Control, Devices, Events, Groups, Monitors, Snapshots, Stream, System,
};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "User_Roles")]
pub struct Model {
    #[sea_orm(column_name = "Id", primary_key)]
    pub id: u32,
    #[sea_orm(column_name = "Name", unique)]
    pub name: String,
    #[sea_orm(column_name = "Description", column_type = "Text", nullable)]
    pub description: Option<String>,
    #[sea_orm(column_name = "Stream")]
    pub stream: Stream,
    #[sea_orm(column_name = "Events")]
    pub events: Events,
    #[sea_orm(column_name = "Control")]
    pub control: Control,
    #[sea_orm(column_name = "Monitors")]
    pub monitors: Monitors,
    #[sea_orm(column_name = "Groups")]
    pub groups: Groups,
    #[sea_orm(column_name = "Devices")]
    pub devices: Devices,
    #[sea_orm(column_name = "Snapshots")]
    pub snapshots: Snapshots,
    #[sea_orm(column_name = "System")]
    pub system: System,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
