//! Is ZoneMinder's schema as new as this build expects (#147)?
//!
//! zm-api's queries name the columns of the latest ZoneMinder version it
//! mirrors. Against an older schema they fail: every monitor read 500s, and
//! in takeover mode the supervisor can't list monitors, so capture stops.
//! When the schema is behind, zm-api refuses to serve, doesn't take over
//! the daemons and doesn't migrate ZoneMinder's tables by itself. The
//! operator upgrades with `zm-api-db up` (or `bridge` for a database not yet
//! on the migration system).

use sea_orm::{DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;

use super::legacy_bridge::{chain::CUTOVER_ZM_VERSION, version_lt};
use super::stamp::BASELINE_VERSION;
use super::stamp::{detect_version, migration_recorded, table_exists};
use super::Migrator;

/// Where the database stands against this build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaState {
    /// Up to date, newer, or empty (the baseline creates it).
    Current,
    /// Older than the ZoneMinder version this build mirrors.
    Behind {
        /// `ZM_DYN_DB_VERSION`, or `None` when the database doesn't say.
        at: Option<String>,
        /// The ZoneMinder upgrade migrations still to run.
        pending: Vec<String>,
    },
}

impl SchemaState {
    /// The message logged at startup and returned with each 503.
    pub fn explain(&self) -> Option<String> {
        let SchemaState::Behind { at, pending } = self else {
            return None;
        };
        let at = at.as_deref().unwrap_or("an unknown version");
        let how = if pending.is_empty() {
            "run `zm-api-db bridge` to bring it onto the migration system"
        } else {
            "run `zm-api-db up` to upgrade it"
        };
        Some(format!(
            "the ZoneMinder database is at {at}, older than the {CUTOVER_ZM_VERSION} this zm-api \
             needs; {how}, then restart zm-api"
        ))
    }
}

/// The ZoneMinder mirrors among `pending` migration names (zm-api's own
/// tables are created at startup and never make a database "behind").
fn upstream_only(pending: impl IntoIterator<Item = String>) -> Vec<String> {
    pending
        .into_iter()
        .filter(|n| n.starts_with("zm_"))
        .collect()
}

/// Whether a ZoneMinder version is older than the one this build mirrors.
fn older(version: &str) -> bool {
    version_lt(version, CUTOVER_ZM_VERSION)
}

/// Behind means ZoneMinder's schema is older than the mirrored version and
/// nothing has brought it forward:
///
/// - ZoneMinder's own version (`ZM_DYN_DB_VERSION`, else the layout) at or
///   past the mirrored one is current, whatever zm-api has recorded
///   (ZoneMinder upgraded by its own packages; startup records the
///   re-runnable mirrors).
/// - On the migration system, it is behind while ZoneMinder mirrors are
///   pending; once `up` has run them it is current, though the mirrors
///   leave `ZM_DYN_DB_VERSION` as it was.
/// - Off it, an older schema needs `bridge`.
pub async fn check(conn: &DatabaseConnection) -> Result<SchemaState, DbErr> {
    if !table_exists(conn, "Monitors").await? {
        return Ok(SchemaState::Current); // fresh: the baseline creates everything
    }
    let version = detect_version(conn).await?;
    if version.as_deref().is_some_and(|v| !older(v)) {
        return Ok(SchemaState::Current);
    }
    if !migration_recorded(conn, BASELINE_VERSION).await? {
        return Ok(SchemaState::Behind {
            at: version,
            pending: Vec::new(),
        });
    }
    let pending = upstream_only(
        Migrator::get_pending_migrations(conn)
            .await?
            .iter()
            .map(|m| m.name().to_string()),
    );
    Ok(if pending.is_empty() {
        SchemaState::Current
    } else {
        SchemaState::Behind {
            at: version,
            pending,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_zoneminder_mirrors_count_as_behind() {
        let names = ["m20260625_000001_create_event_synopsis", "zm_1_39_41"].map(String::from);
        assert_eq!(upstream_only(names), ["zm_1_39_41"]);
    }

    #[test]
    fn versions_below_the_mirrored_one_are_older() {
        assert!(older("1.39.22"));
        assert!(!older(CUTOVER_ZM_VERSION));
        assert!(!older("1.39.99"));
    }

    #[test]
    fn the_message_names_the_command() {
        let up = SchemaState::Behind {
            at: Some("1.39.22".into()),
            pending: vec!["zm_1_39_23".into()],
        };
        let m = up.explain().unwrap();
        assert!(m.contains("1.39.22") && m.contains("zm-api-db up"), "{m}");
        let bridge = SchemaState::Behind {
            at: None,
            pending: vec![],
        };
        assert!(bridge.explain().unwrap().contains("zm-api-db bridge"));
        assert_eq!(SchemaState::Current.explain(), None);
    }
}
