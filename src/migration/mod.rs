//! Database migrations for zm_api-owned tables.
//!
//! Note: ZoneMinder's schema is managed externally. These migrations are only
//! for tables owned by zm_api itself (if any are needed in the future).
//!
//! SeaORM supports both MySQL/MariaDB and PostgreSQL - migrations here should
//! use portable SQL or conditional logic for database-specific syntax.

pub use sea_orm_migration::prelude::*;

mod m20260625_000001_create_event_synopsis;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // zm_api-owned tables only. ZoneMinder's own schema is created and
            // managed by ZoneMinder itself — never migrated from here.
            Box::new(m20260625_000001_create_event_synopsis::Migration),
        ]
    }
}
