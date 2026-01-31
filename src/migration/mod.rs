//! Database migrations for zm_api-owned tables.
//!
//! Note: ZoneMinder's schema is managed externally. These migrations are only
//! for tables owned by zm_api itself (if any are needed in the future).
//!
//! SeaORM supports both MySQL/MariaDB and PostgreSQL - migrations here should
//! use portable SQL or conditional logic for database-specific syntax.

pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // No zm_api-specific migrations yet.
            // ZoneMinder's schema is created/managed by ZoneMinder itself.
        ]
    }
}
