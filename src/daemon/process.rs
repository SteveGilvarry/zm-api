//! Process state machine and managed process types.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::process::Child;
use utoipa::ToSchema;

/// State of a managed daemon process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProcessState {
    /// Process is not running
    Stopped,
    /// Process is being started
    Starting,
    /// Process is running normally
    Running,
    /// Process is being stopped (SIGTERM sent)
    Stopping,
    /// Process failed to start or crashed
    Failed,
    /// Process is waiting to restart (backoff period)
    Restarting,
}

impl std::fmt::Display for ProcessState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessState::Stopped => write!(f, "stopped"),
            ProcessState::Starting => write!(f, "starting"),
            ProcessState::Running => write!(f, "running"),
            ProcessState::Stopping => write!(f, "stopping"),
            ProcessState::Failed => write!(f, "failed"),
            ProcessState::Restarting => write!(f, "restarting"),
        }
    }
}

/// A managed daemon process.
#[derive(Debug)]
pub struct ManagedProcess {
    /// Unique identifier (e.g., "zmc -m 1" or "zmfilter.pl")
    pub id: String,
    /// Display name for the daemon
    pub name: String,
    /// Command to execute
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Current process state
    pub state: ProcessState,
    /// Process ID when running
    pub pid: Option<u32>,
    /// Tokio child process handle (not serializable)
    child: Option<Child>,
    /// Time of last state change
    pub last_state_change: Instant,
    /// Time process started running (for uptime calculation)
    pub started_at: Option<Instant>,
    /// Number of restart attempts since last successful start
    pub restart_count: u32,
    /// Current backoff delay for restarts
    pub current_backoff: Duration,
    /// Whether this daemon should auto-restart on failure
    pub auto_restart: bool,
    /// Optional monitor ID for zmc instances
    pub monitor_id: Option<u32>,
    /// Time when SIGTERM was sent (for timeout tracking)
    pub term_sent_at: Option<Instant>,
}

impl ManagedProcess {
    /// Create a new managed process.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        command: impl Into<String>,
        args: Vec<String>,
        auto_restart: bool,
        monitor_id: Option<u32>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            command: command.into(),
            args,
            state: ProcessState::Stopped,
            pid: None,
            child: None,
            last_state_change: Instant::now(),
            started_at: None,
            restart_count: 0,
            current_backoff: Duration::from_secs(5),
            auto_restart,
            monitor_id,
            term_sent_at: None,
        }
    }

    /// Create a managed process for a ZoneMinder capture daemon (zmc).
    pub fn for_monitor(monitor_id: u32, device: Option<&str>) -> Self {
        let (id, args) = if let Some(dev) = device {
            (
                format!("zmc -d {}", dev),
                vec!["-d".to_string(), dev.to_string()],
            )
        } else {
            (
                format!("zmc -m {}", monitor_id),
                vec!["-m".to_string(), monitor_id.to_string()],
            )
        };

        Self::new(
            id,
            format!("zmc[{}]", monitor_id),
            "zmc",
            args,
            true,
            Some(monitor_id),
        )
    }

    /// Set the child process handle and update state to Running.
    pub fn set_child(&mut self, child: Child) {
        self.pid = child.id();
        self.child = Some(child);
        self.set_state(ProcessState::Running);
        self.started_at = Some(Instant::now());
        self.restart_count = 0;
    }

    /// Take ownership of the child process handle.
    pub fn take_child(&mut self) -> Option<Child> {
        self.child.take()
    }

    /// Check if the process has a child handle.
    pub fn has_child(&self) -> bool {
        self.child.is_some()
    }

    /// Get a mutable reference to the child process.
    pub fn child_mut(&mut self) -> Option<&mut Child> {
        self.child.as_mut()
    }

    /// Update the process state.
    pub fn set_state(&mut self, state: ProcessState) {
        self.state = state;
        self.last_state_change = Instant::now();

        // Clear PID and child when stopped/failed
        if matches!(state, ProcessState::Stopped | ProcessState::Failed) {
            self.pid = None;
            self.child = None;
            self.started_at = None;
        }
    }

    /// Get the uptime of the process in seconds, if running.
    pub fn uptime(&self) -> Option<Duration> {
        if self.state == ProcessState::Running {
            self.started_at.map(|start| start.elapsed())
        } else {
            None
        }
    }

    /// Get the time since the last state change.
    pub fn time_in_state(&self) -> Duration {
        self.last_state_change.elapsed()
    }

    /// Check if the process is in a running state.
    pub fn is_running(&self) -> bool {
        self.state == ProcessState::Running
    }

    /// Check if the process can be started.
    pub fn can_start(&self) -> bool {
        matches!(
            self.state,
            ProcessState::Stopped | ProcessState::Failed | ProcessState::Restarting
        )
    }

    /// Check if the process can be stopped.
    pub fn can_stop(&self) -> bool {
        matches!(
            self.state,
            ProcessState::Running | ProcessState::Starting | ProcessState::Restarting
        )
    }

    /// Mark that SIGTERM was sent.
    pub fn mark_term_sent(&mut self) {
        self.term_sent_at = Some(Instant::now());
        self.set_state(ProcessState::Stopping);
    }

    /// Check if the SIGTERM timeout has expired.
    pub fn term_timeout_expired(&self, timeout: Duration) -> bool {
        self.term_sent_at
            .map(|sent| sent.elapsed() >= timeout)
            .unwrap_or(false)
    }

    /// Prepare for restart with backoff.
    pub fn prepare_restart(&mut self, min_backoff: Duration, max_backoff: Duration) {
        self.restart_count += 1;
        self.set_state(ProcessState::Restarting);

        // Calculate exponential backoff
        let backoff_secs = min_backoff.as_secs() * 2u64.pow(self.restart_count.min(10));
        self.current_backoff = Duration::from_secs(backoff_secs.min(max_backoff.as_secs()));
    }

    /// Check if the backoff period has elapsed.
    pub fn backoff_elapsed(&self) -> bool {
        self.time_in_state() >= self.current_backoff
    }

    /// Reset backoff on successful start.
    pub fn reset_backoff(&mut self, min_backoff: Duration) {
        self.restart_count = 0;
        self.current_backoff = min_backoff;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_process() {
        let process = ManagedProcess::new(
            "test-daemon",
            "Test Daemon",
            "/usr/bin/test",
            vec!["-v".to_string()],
            true,
            None,
        );

        assert_eq!(process.id, "test-daemon");
        assert_eq!(process.name, "Test Daemon");
        assert_eq!(process.command, "/usr/bin/test");
        assert_eq!(process.args, vec!["-v"]);
        assert_eq!(process.state, ProcessState::Stopped);
        assert!(process.auto_restart);
        assert!(process.pid.is_none());
        assert!(process.monitor_id.is_none());
    }

    #[test]
    fn test_for_monitor_with_id() {
        let process = ManagedProcess::for_monitor(5, None);

        assert_eq!(process.id, "zmc -m 5");
        assert_eq!(process.name, "zmc[5]");
        assert_eq!(process.command, "zmc");
        assert_eq!(process.args, vec!["-m", "5"]);
        assert_eq!(process.monitor_id, Some(5));
    }

    #[test]
    fn test_for_monitor_with_device() {
        let process = ManagedProcess::for_monitor(1, Some("/dev/video0"));

        assert_eq!(process.id, "zmc -d /dev/video0");
        assert_eq!(process.args, vec!["-d", "/dev/video0"]);
    }

    #[test]
    fn test_state_transitions() {
        let mut process = ManagedProcess::new("test", "test", "test", vec![], true, None);

        assert!(process.can_start());
        assert!(!process.can_stop());

        process.set_state(ProcessState::Running);
        assert!(!process.can_start());
        assert!(process.can_stop());
        assert!(process.is_running());

        process.set_state(ProcessState::Stopped);
        assert!(process.can_start());
        assert!(!process.is_running());
    }

    #[test]
    fn test_backoff_calculation() {
        let mut process = ManagedProcess::new("test", "test", "test", vec![], true, None);
        let min = Duration::from_secs(5);
        let max = Duration::from_secs(900);

        process.prepare_restart(min, max);
        assert_eq!(process.restart_count, 1);
        assert_eq!(process.current_backoff, Duration::from_secs(10)); // 5 * 2^1

        process.prepare_restart(min, max);
        assert_eq!(process.restart_count, 2);
        assert_eq!(process.current_backoff, Duration::from_secs(20)); // 5 * 2^2

        // Test max cap
        for _ in 0..20 {
            process.prepare_restart(min, max);
        }
        assert!(process.current_backoff <= max);
    }

    #[test]
    fn test_process_state_display() {
        assert_eq!(ProcessState::Stopped.to_string(), "stopped");
        assert_eq!(ProcessState::Running.to_string(), "running");
        assert_eq!(ProcessState::Failed.to_string(), "failed");
    }
}
