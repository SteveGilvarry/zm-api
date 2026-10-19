//! zm-api-owned `media_jobs` table: the media job queue (FILTERS_PLAN.md,
//! Subsystem 3). Hand-written (see
//! `src/migration/m20261018_000001_create_media_jobs.rs`). Times are UTC.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "media_jobs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub kind: String,
    pub event_id: u64,
    pub monitor_id: u32,
    #[sea_orm(column_type = "Text")]
    pub params: String,
    pub status: String,
    pub output: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub error: Option<String>,
    pub created_by: Option<u32>,
    pub created_at: DateTime,
    pub started_at: Option<DateTime>,
    pub finished_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
