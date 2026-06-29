//! zm_api-owned `monitor_pipeline` table — the zm-next processing plugin graph.
//!
//! Unlike the rest of `src/entity/`, this is **not** generated from ZoneMinder's
//! schema: it is a zm-api-owned table created by the migration in
//! `src/migration/`. One row per monitor holds the editable processing plugin
//! graph (the `{id,kind,cfg,children}` tree from `decode` downward) that
//! ZoneMinder has no schema for. Capture, credentials, the `store` node, and
//! zones are NOT stored here — they are re-derived from `Monitors`/`Zones` and
//! composed in at spawn, so this table holds no secrets and never competes with
//! ZoneMinder as a source of truth. Columns are snake_case (our own naming).

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "monitor_pipeline")]
pub struct Model {
    /// Logical FK to `Monitors.Id`; one processing graph per monitor.
    #[sea_orm(primary_key, auto_increment = false)]
    pub monitor_id: u32,
    /// The processing plugin graph as a JSON document. Validated by zm-api on
    /// write (well-formed tree, known plugin `kind`s, no capture/creds nodes) and
    /// deeply validated by zm-next at spawn.
    #[sea_orm(column_type = "Text")]
    pub graph_json: String,
    /// Document/schema version, so the graph shape can evolve additively.
    pub version: u32,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

/// `monitor_id` is a *logical* FK to `Monitors.Id`. No hard DB constraint is
/// created — zm-api does not own ZoneMinder's `Monitors` table — but the relation
/// lets queries join through to the owning monitor.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::monitors::Entity",
        from = "Column::MonitorId",
        to = "super::monitors::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Monitors,
}

impl Related<super::monitors::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Monitors.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
