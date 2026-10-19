//! `SeaORM` Entity for ZoneMinder 1.39's `Role_Monitors_Permissions`: a role's
//! per-monitor permission. Curated by hand to match `zm_create.sql.in`.

use super::sea_orm_active_enums::Permission;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "Role_Monitors_Permissions")]
pub struct Model {
    #[sea_orm(column_name = "Id", primary_key)]
    pub id: u32,
    #[sea_orm(column_name = "RoleId")]
    pub role_id: u32,
    #[sea_orm(column_name = "MonitorId")]
    pub monitor_id: u32,
    #[sea_orm(column_name = "Permission")]
    pub permission: Permission,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
