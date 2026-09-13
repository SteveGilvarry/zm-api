//! One SeaORM migration per upstream `db/zm_update-1.39.x.sql`, from 1.39.2
//! (the baseline is the 1.39.1 schema) to the latest vendored update. Each
//! module's doc comment names the file it mirrors; `helpers` holds the
//! portable existence probes and type helpers. See docs/DB_VERSIONING_PLAN.md.
//!
//! Existing MySQL installs are upgraded by the raw chain in `legacy_bridge`
//! and then stamped with every migration here at or below the version they
//! reached; fresh installs on either backend run these directly.

use sea_orm_migration::prelude::*;

pub mod helpers;

mod controls;
pub(crate) mod triggers_1_39_26;
mod zm_1_39_10;
mod zm_1_39_12;
mod zm_1_39_13;
mod zm_1_39_14;
mod zm_1_39_15;
mod zm_1_39_16;
mod zm_1_39_17;
mod zm_1_39_18;
mod zm_1_39_19;
mod zm_1_39_2;
mod zm_1_39_20;
mod zm_1_39_21;
mod zm_1_39_22;
mod zm_1_39_23;
mod zm_1_39_24;
mod zm_1_39_25;
mod zm_1_39_26;
mod zm_1_39_27;
mod zm_1_39_28;
mod zm_1_39_29;
mod zm_1_39_3;
mod zm_1_39_30;
mod zm_1_39_31;
mod zm_1_39_32;
mod zm_1_39_33;
mod zm_1_39_4;
mod zm_1_39_5;
mod zm_1_39_8;
mod zm_1_39_9;

/// The ZoneMinder version each migration brings the schema to, in order.
pub const VERSIONS: &[&str] = &[
    "1.39.2", "1.39.3", "1.39.4", "1.39.5", "1.39.8", "1.39.9", "1.39.10", "1.39.12", "1.39.13",
    "1.39.14", "1.39.15", "1.39.16", "1.39.17", "1.39.18", "1.39.19", "1.39.20", "1.39.21",
    "1.39.22", "1.39.23", "1.39.24", "1.39.25", "1.39.26", "1.39.27", "1.39.28", "1.39.29",
    "1.39.30", "1.39.31", "1.39.32", "1.39.33",
];

/// Migration name for a version: `1.39.2` → `zm_1_39_2` (the module file stem,
/// which is what `DeriveMigrationName` records).
pub fn migration_name(version: &str) -> String {
    format!("zm_{}", version.replace('.', "_"))
}

/// The MySQL trigger set the latest mirrored update leaves behind — what a
/// bridged install must converge to.
pub(crate) fn current_mysql_triggers() -> Vec<(&'static str, &'static str)> {
    triggers_1_39_26::mysql_triggers()
}

/// The migrations, in chain order.
pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(zm_1_39_2::Migration),
        Box::new(zm_1_39_3::Migration),
        Box::new(zm_1_39_4::Migration),
        Box::new(zm_1_39_5::Migration),
        Box::new(zm_1_39_8::Migration),
        Box::new(zm_1_39_9::Migration),
        Box::new(zm_1_39_10::Migration),
        Box::new(zm_1_39_12::Migration),
        Box::new(zm_1_39_13::Migration),
        Box::new(zm_1_39_14::Migration),
        Box::new(zm_1_39_15::Migration),
        Box::new(zm_1_39_16::Migration),
        Box::new(zm_1_39_17::Migration),
        Box::new(zm_1_39_18::Migration),
        Box::new(zm_1_39_19::Migration),
        Box::new(zm_1_39_20::Migration),
        Box::new(zm_1_39_21::Migration),
        Box::new(zm_1_39_22::Migration),
        Box::new(zm_1_39_23::Migration),
        Box::new(zm_1_39_24::Migration),
        Box::new(zm_1_39_25::Migration),
        Box::new(zm_1_39_26::Migration),
        Box::new(zm_1_39_27::Migration),
        Box::new(zm_1_39_28::Migration),
        Box::new(zm_1_39_29::Migration),
        Box::new(zm_1_39_30::Migration),
        Box::new(zm_1_39_31::Migration),
        Box::new(zm_1_39_32::Migration),
        Box::new(zm_1_39_33::Migration),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The version list and the migration list are the same thing in two
    /// shapes; stamping relies on `migration_name(version)` naming a real
    /// migration, in order.
    #[test]
    fn versions_and_migrations_line_up() {
        let names: Vec<String> = migrations().iter().map(|m| m.name().to_string()).collect();
        let expected: Vec<String> = VERSIONS.iter().map(|v| migration_name(v)).collect();
        assert_eq!(names, expected);
    }

    #[test]
    fn versions_are_strictly_ascending() {
        let key = |v: &str| {
            v.split('.')
                .map(|p| p.parse::<u32>().unwrap())
                .collect::<Vec<_>>()
        };
        for w in VERSIONS.windows(2) {
            assert!(key(w[0]) < key(w[1]), "{} before {}", w[0], w[1]);
        }
    }
}
