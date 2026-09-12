//! Which `Servers` row is this host.
//!
//! ZoneMinder's own daemons learn their server identity from `zm.conf`:
//! `ZM_SERVER_ID` directly, else `ZM_SERVER_HOST` looked up against
//! `Servers.Name`. zm-api used to hard-code `None` for the daemon manager and
//! read an environment variable nothing sets for the stats job (#75, #100),
//! so on a multi-server install every host started every monitor's zmc and
//! logged stats under `ServerId = 0`. This resolves it the way ZoneMinder
//! does, with the machine hostname as a last resort, and `None` on a
//! single-server install (no `Servers` rows), where the per-server gates are
//! meant to be ignored.

use sea_orm::{DatabaseConnection, EntityTrait};
use tracing::{info, warn};

use crate::configure::zmconf::ZmConfig;
use crate::entity::servers;

/// One `Servers` row, reduced to what identity matching needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerCandidate {
    pub id: u32,
    pub name: String,
    pub hostname: Option<String>,
}

/// Pick this host's server id from the available evidence, in ZoneMinder's
/// order of precedence: an explicit `ZM_SERVER_ID`, then `ZM_SERVER_HOST`
/// matched against `Servers`, then the machine's own hostname. A name matches
/// a row when it equals `Name` or `Hostname` case-insensitively, or when the
/// short labels (before the first `.`) do — `cam1` and `cam1.lan` are the same
/// box, and both spellings turn up in `Servers` tables in the wild.
pub fn pick_server_id(
    conf_id: Option<&str>,
    conf_host: Option<&str>,
    machine_hostname: Option<&str>,
    servers: &[ServerCandidate],
) -> Option<u32> {
    if let Some(id) = conf_id.and_then(|v| v.trim().parse::<u32>().ok()) {
        if id != 0 {
            return Some(id);
        }
    }
    conf_host
        .and_then(|h| match_server(h, servers))
        .or_else(|| machine_hostname.and_then(|h| match_server(h, servers)))
}

fn match_server(host: &str, servers: &[ServerCandidate]) -> Option<u32> {
    let host = host.trim();
    if host.is_empty() {
        return None;
    }
    fn same(a: &str, b: &str) -> bool {
        a.eq_ignore_ascii_case(b)
    }
    fn short(s: &str) -> &str {
        s.split('.').next().unwrap_or(s)
    }

    let exact = servers
        .iter()
        .find(|s| same(&s.name, host) || s.hostname.as_deref().is_some_and(|h| same(h, host)));
    if let Some(s) = exact {
        return Some(s.id);
    }
    servers
        .iter()
        .find(|s| {
            same(short(&s.name), short(host))
                || s.hostname
                    .as_deref()
                    .is_some_and(|h| same(short(h), short(host)))
        })
        .map(|s| s.id)
}

/// Resolve this host's `Servers.Id` from `zm.conf`, the `Servers` table and
/// the machine hostname. `None` means single-server (or unresolvable, which
/// is logged so a multi-server operator can see why the gates are open).
pub async fn resolve_server_id(db: &DatabaseConnection) -> Option<u32> {
    let rows = match servers::Entity::find().all(db).await {
        Ok(rows) => rows,
        Err(e) => {
            warn!("Could not read Servers to resolve this host's server id: {e}");
            return None;
        }
    };
    if rows.is_empty() {
        return None; // single-server install
    }
    let candidates: Vec<ServerCandidate> = rows
        .into_iter()
        .map(|s| ServerCandidate {
            id: s.id,
            name: s.name,
            hostname: s.hostname,
        })
        .collect();

    let conf = ZmConfig::load();
    let machine = nix::unistd::gethostname()
        .ok()
        .and_then(|h| h.into_string().ok());

    let picked = pick_server_id(
        conf.get("ZM_SERVER_ID"),
        conf.get("ZM_SERVER_HOST"),
        machine.as_deref(),
        &candidates,
    );
    match picked {
        Some(id) => info!("This host is Servers.Id {id}"),
        None => warn!(
            "Servers has {} rows but none matches ZM_SERVER_ID, ZM_SERVER_HOST or hostname {:?}; \
             per-server daemon gates and ServerId filtering are off",
            candidates.len(),
            machine
        ),
    }
    picked
}

#[cfg(test)]
mod tests {
    use super::*;

    fn servers() -> Vec<ServerCandidate> {
        vec![
            ServerCandidate {
                id: 1,
                name: "nvr-a".into(),
                hostname: Some("nvr-a.lan".into()),
            },
            ServerCandidate {
                id: 2,
                name: "nvr-b".into(),
                hostname: None,
            },
        ]
    }

    #[test]
    fn explicit_zm_server_id_wins() {
        assert_eq!(
            pick_server_id(Some("2"), Some("nvr-a"), Some("nvr-a"), &servers()),
            Some(2)
        );
    }

    #[test]
    fn zero_or_garbage_zm_server_id_is_ignored() {
        assert_eq!(
            pick_server_id(Some("0"), Some("nvr-a"), None, &servers()),
            Some(1)
        );
        assert_eq!(
            pick_server_id(Some("x"), None, Some("nvr-b"), &servers()),
            Some(2)
        );
    }

    #[test]
    fn zm_server_host_matches_name_or_hostname_case_insensitively() {
        assert_eq!(
            pick_server_id(None, Some("NVR-B"), None, &servers()),
            Some(2)
        );
        assert_eq!(
            pick_server_id(None, Some("nvr-a.lan"), None, &servers()),
            Some(1)
        );
    }

    #[test]
    fn machine_hostname_is_the_fallback_and_short_labels_match() {
        assert_eq!(
            pick_server_id(None, None, Some("nvr-b.example.com"), &servers()),
            Some(2)
        );
        assert_eq!(
            pick_server_id(None, None, Some("nvr-a"), &servers()),
            Some(1)
        );
    }

    #[test]
    fn no_match_and_no_rows_are_single_server() {
        assert_eq!(pick_server_id(None, None, Some("other"), &servers()), None);
        assert_eq!(pick_server_id(None, Some("nvr-a"), None, &[]), None);
    }
}
