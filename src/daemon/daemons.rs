//! ZoneMinder daemon definitions.

/// Definition of a ZoneMinder daemon.
#[derive(Debug, Clone)]
pub struct DaemonDefinition {
    /// Unique name for the daemon
    pub name: &'static str,
    /// Command to execute
    pub command: &'static str,
    /// Description of what the daemon does
    pub description: &'static str,
    /// Whether this daemon should auto-restart on failure
    pub auto_restart: bool,
    /// Whether only one instance of this daemon is allowed
    pub singleton: bool,
    /// Whether this daemon requires database access
    pub requires_db: bool,
    /// Startup priority (lower = earlier)
    pub priority: u8,
}

/// All ZoneMinder daemons managed by zmdc.
pub const ZM_DAEMONS: &[DaemonDefinition] = &[
    DaemonDefinition {
        name: "zmfilter",
        command: "zmfilter.pl",
        description: "Event filter daemon - handles event filtering and deletion",
        auto_restart: true,
        singleton: true,
        requires_db: true,
        priority: 10,
    },
    DaemonDefinition {
        name: "zmaudit",
        command: "zmaudit.pl",
        description: "Audit daemon - verifies event filesystem consistency",
        auto_restart: true,
        singleton: true,
        requires_db: true,
        priority: 20,
    },
    DaemonDefinition {
        name: "zmtrigger",
        command: "zmtrigger.pl",
        description: "Trigger daemon - handles external alarm triggers",
        auto_restart: true,
        singleton: true,
        requires_db: true,
        priority: 30,
    },
    DaemonDefinition {
        name: "zmcontrol",
        command: "zmcontrol.pl",
        description: "PTZ control daemon - handles camera pan/tilt/zoom",
        auto_restart: true,
        singleton: false, // One per controllable monitor
        requires_db: true,
        priority: 40,
    },
    DaemonDefinition {
        name: "zmtrack",
        command: "zmtrack.pl",
        description: "Tracking daemon - handles motion tracking",
        auto_restart: true,
        singleton: false, // One per tracking monitor
        requires_db: true,
        priority: 50,
    },
    DaemonDefinition {
        name: "zmwatch",
        command: "zmwatch.pl",
        description: "Watch daemon - monitors capture daemons and restarts if needed",
        auto_restart: true,
        singleton: true,
        requires_db: true,
        priority: 60,
    },
    DaemonDefinition {
        name: "zmstats",
        command: "zmstats.pl",
        description: "Stats daemon - collects system statistics",
        auto_restart: true,
        singleton: true,
        requires_db: true,
        priority: 70,
    },
    DaemonDefinition {
        name: "zmtelemetry",
        command: "zmtelemetry.pl",
        description: "Telemetry daemon - sends anonymous usage stats",
        auto_restart: true,
        singleton: true,
        requires_db: true,
        priority: 80,
    },
    DaemonDefinition {
        name: "zmeventnotification",
        command: "zmeventnotification.pl",
        description: "Event notification daemon - sends push notifications",
        auto_restart: true,
        singleton: true,
        requires_db: true,
        priority: 90,
    },
    DaemonDefinition {
        name: "zmc",
        command: "zmc",
        description: "Capture daemon - captures video from cameras",
        auto_restart: true,
        singleton: false, // One per monitor (or device)
        requires_db: false,
        priority: 5, // Start early
    },
    DaemonDefinition {
        name: "zma",
        command: "zma",
        description: "Analysis daemon - analyzes video for motion",
        auto_restart: true,
        singleton: false, // One per monitor
        requires_db: false,
        priority: 6,
    },
];

impl DaemonDefinition {
    /// Find a daemon definition by name.
    pub fn find_by_name(name: &str) -> Option<&'static DaemonDefinition> {
        ZM_DAEMONS.iter().find(|d| d.name == name)
    }

    /// Find a daemon definition by command.
    pub fn find_by_command(command: &str) -> Option<&'static DaemonDefinition> {
        ZM_DAEMONS.iter().find(|d| d.command == command)
    }

    /// Get all singleton daemons (for system startup).
    pub fn singletons() -> impl Iterator<Item = &'static DaemonDefinition> {
        ZM_DAEMONS.iter().filter(|d| d.singleton)
    }

    /// Get daemons sorted by startup priority.
    pub fn by_priority() -> Vec<&'static DaemonDefinition> {
        let mut daemons: Vec<_> = ZM_DAEMONS.iter().collect();
        daemons.sort_by_key(|d| d.priority);
        daemons
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_by_name() {
        let daemon = DaemonDefinition::find_by_name("zmfilter");
        assert!(daemon.is_some());
        assert_eq!(daemon.unwrap().command, "zmfilter.pl");
    }

    #[test]
    fn test_find_by_command() {
        let daemon = DaemonDefinition::find_by_command("zmc");
        assert!(daemon.is_some());
        assert_eq!(daemon.unwrap().name, "zmc");
    }

    #[test]
    fn test_find_nonexistent() {
        assert!(DaemonDefinition::find_by_name("nonexistent").is_none());
        assert!(DaemonDefinition::find_by_command("nonexistent").is_none());
    }

    #[test]
    fn test_singletons() {
        let singletons: Vec<_> = DaemonDefinition::singletons().collect();
        // zmfilter, zmaudit, zmtrigger, zmwatch, zmstats, zmtelemetry, zmeventnotification
        assert!(singletons.iter().all(|d| d.singleton));
        assert!(singletons.iter().any(|d| d.name == "zmfilter"));
        assert!(singletons.iter().any(|d| d.name == "zmwatch"));
    }

    #[test]
    fn test_by_priority() {
        let ordered = DaemonDefinition::by_priority();
        // zmc should come first (priority 5)
        assert_eq!(ordered[0].name, "zmc");
        // Then zma (priority 6)
        assert_eq!(ordered[1].name, "zma");
    }

    #[test]
    fn test_all_daemons_have_unique_names() {
        let mut names: Vec<_> = ZM_DAEMONS.iter().map(|d| d.name).collect();
        let len_before = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), len_before, "Daemon names must be unique");
    }
}
