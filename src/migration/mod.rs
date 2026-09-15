//! Database migrations.
//!
//! As of migration system phase 1 (issue #11) zm-api owns the full ZoneMinder
//! schema: `m00000000_000001_zm_baseline` creates every ZM table for fresh
//! installs. Existing ZoneMinder databases must NOT run the baseline — the
//! upgrade bridge (phase 2) walks the legacy zm_update-*.sql chain to the
//! cutover release and then stamps the baseline as applied.
//!
//! SeaORM supports both MySQL/MariaDB and PostgreSQL - migrations here should
//! use portable SQL or conditional logic for database-specific syntax.

pub use sea_orm_migration::prelude::*;

pub mod legacy_bridge;
mod m00000000_000001_zm_baseline;
mod m20260625_000001_create_event_synopsis;
mod m20260627_000001_create_monitor_pipeline;
mod m20260914_000001_create_zmnext_secret;
pub mod stamp;
pub mod upstream;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut all: Vec<Box<dyn MigrationTrait>> =
            vec![Box::new(m00000000_000001_zm_baseline::Migration)];
        // Upstream's 1.39 chain, mirrored (see upstream/mod.rs), then
        // zm-api's own tables on top.
        all.extend(upstream::migrations());
        all.push(Box::new(m20260625_000001_create_event_synopsis::Migration));
        all.push(Box::new(
            m20260627_000001_create_monitor_pipeline::Migration,
        ));
        all.push(Box::new(m20260914_000001_create_zmnext_secret::Migration));
        all
    }
}
