//! `SeaORM` Entity for ZoneMinder 1.39's `Notifications`: push token
//! registrations (FCM/APNs/web) for zmeventnotification-style push. Curated
//! by hand to match `zm_create.sql.in`; the enum columns are strings, typed
//! in the DTOs. Datetimes are server-local, like ZoneMinder's others; `UpdatedOn`
//! is a TIMESTAMP, which the driver reads as UTC.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "Notifications")]
pub struct Model {
    #[sea_orm(column_name = "Id", primary_key)]
    pub id: u32,
    #[sea_orm(column_name = "UserId")]
    pub user_id: Option<u32>,
    #[sea_orm(column_name = "Token")]
    pub token: String,
    #[sea_orm(column_name = "Platform")]
    pub platform: String,
    #[sea_orm(column_name = "MonitorList", column_type = "Text", nullable)]
    pub monitor_list: Option<String>,
    #[sea_orm(column_name = "Interval")]
    pub interval: u32,
    #[sea_orm(column_name = "PushState")]
    pub push_state: String,
    #[sea_orm(column_name = "AppVersion")]
    pub app_version: Option<String>,
    #[sea_orm(column_name = "Profile")]
    pub profile: Option<String>,
    #[sea_orm(column_name = "BadgeCount")]
    pub badge_count: i32,
    #[sea_orm(column_name = "LastNotifiedAt")]
    pub last_notified_at: Option<DateTime>,
    #[sea_orm(column_name = "CreatedOn")]
    pub created_on: Option<DateTime>,
    #[sea_orm(column_name = "UpdatedOn")]
    pub updated_on: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
