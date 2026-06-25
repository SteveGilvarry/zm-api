//! zm_api-owned `event_synopsis` table — render state for a motion synopsis.
//!
//! Unlike the rest of `src/entity/`, this is **not** generated from
//! ZoneMinder's schema: it is a zm-api-owned table created by the migration in
//! `src/migration/`. One row tracks the manifest + render lifecycle for a single
//! `review_assets` (0x0306) event. Columns are snake_case (our own naming), so
//! no `column_name` overrides are needed.

use super::sea_orm_active_enums::SynopsisStatus;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "event_synopsis")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u64,
    /// Source event id, when the recording_opening→assign handshake reconciled
    /// it. `None` until then — keyed by `(monitor_id, clip_token)` meanwhile.
    pub event_id: Option<u64>,
    pub monitor_id: u32,
    pub clip_token: String,
    /// The raw `review_assets` manifest JSON, stored verbatim for the renderer.
    #[sea_orm(column_type = "Text")]
    pub manifest_json: String,
    /// Resolved absolute directory holding the cutout/plate assets.
    #[sea_orm(column_type = "Text")]
    pub asset_dir: String,
    pub status: SynopsisStatus,
    /// Path of the cached rendered artifact (mp4 / still), once `Ready`.
    #[sea_orm(column_type = "Text", nullable)]
    pub rendered_path: Option<String>,
    /// Tube count from the manifest (cheap glance metric for the API).
    pub tube_count: u32,
    /// Source coordinate space the manifest's bboxes/polygons live in.
    pub source_w: u32,
    pub source_h: u32,
    pub created_at: DateTime,
    /// Drives the retention cleanup job. `None` → never expires.
    pub expires_at: Option<DateTime>,
}

/// `event_id` is a *logical* FK to `Events.Id` (nullable until reconciled). No
/// hard DB constraint is created — zm-api does not own the `Events` table and a
/// cross-table constraint would couple our migration to ZoneMinder's schema —
/// but the relation lets queries join through to the source event.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::events::Entity",
        from = "Column::EventId",
        to = "super::events::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Events,
}

impl Related<super::events::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Events.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
