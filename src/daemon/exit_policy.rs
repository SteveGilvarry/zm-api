//! When to stop restarting a zm-next worker.
//!
//! zm-core exits with a fixed status for failures a restart can't fix: a bad
//! command line or a pipeline that doesn't load. Restarting those with backoff
//! forever only fills the log. After [`GIVE_UP_AFTER`] consecutive fast exits
//! with the same such status, the worker is left stopped with a reason until
//! someone starts it again.
//!
//! Two sets of codes are recognised (zm-next `docs/Worker_Control_Protocol.md`,
//! "Exit codes"):
//!
//! * today's zm-core: 1 bad argument, 2 no pipeline in the directory, 3 pipeline
//!   failed to load, 4 plugins failed to load, 5 worker socket failed to start;
//! * Phase 1: 64 bad command line, 69 socket path unusable, 70 internal error.
//!
//! 70, signals and anything unrecognised keep the normal backoff: the doc says
//! a restart can help those.

use std::time::Duration;

/// Consecutive fast exits with the same non-restartable status before giving up.
pub const GIVE_UP_AFTER: u32 = 3;

/// An exit this soon after spawn counts as "fast": the worker never got going.
pub const FAST_EXIT: Duration = Duration::from_secs(10);

/// What a zm-core exit status says about restarting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitClass {
    /// The configuration or command line is wrong; a restart can't fix it.
    Config(&'static str),
    /// The environment needs an operator (socket path, permissions).
    Operator(&'static str),
    /// Worth restarting with backoff.
    Transient,
}

/// Classify a zm-core exit. `code` is `None` when a signal ended the process.
pub fn classify(code: Option<i32>) -> ExitClass {
    match code {
        Some(1) => ExitClass::Config("zm-core rejected its command line (exit 1)"),
        Some(2) => ExitClass::Config("zm-core found no pipeline to load (exit 2)"),
        Some(3) => ExitClass::Config("the pipeline failed to load (exit 3)"),
        Some(4) => ExitClass::Config("a plugin in the pipeline failed to load (exit 4)"),
        Some(64) => ExitClass::Config("zm-core rejected its command line (exit 64)"),
        Some(5) => ExitClass::Operator("the worker socket could not be opened (exit 5)"),
        Some(69) => ExitClass::Operator("the worker socket path is unusable (exit 69)"),
        _ => ExitClass::Transient,
    }
}

/// Running record of a worker's recent exits.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExitHistory {
    /// Status of the most recent exit (`None` for a signal).
    pub last_code: Option<i32>,
    /// Consecutive fast exits with `last_code`, counting only non-restartable ones.
    pub streak: u32,
}

/// Outcome of recording an exit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Restart,
    GiveUp(String),
}

impl ExitHistory {
    /// Record an exit after `uptime` and decide whether to restart.
    pub fn record(&mut self, code: Option<i32>, uptime: Duration) -> Decision {
        let class = classify(code);
        let fast = uptime < FAST_EXIT;
        let counts = fast && !matches!(class, ExitClass::Transient);
        self.streak = if !counts {
            0
        } else if self.streak > 0 && self.last_code == code {
            self.streak + 1
        } else {
            1
        };
        self.last_code = code;

        match class {
            ExitClass::Config(why) | ExitClass::Operator(why) if self.streak >= GIVE_UP_AFTER => {
                Decision::GiveUp(format!(
                    "stopped restarting after {} fast exits: {why}",
                    self.streak
                ))
            }
            _ => Decision::Restart,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const QUICK: Duration = Duration::from_secs(1);
    const LONG: Duration = Duration::from_secs(600);

    #[test]
    fn classifies_both_code_sets() {
        for code in [1, 2, 3, 4, 64] {
            assert!(
                matches!(classify(Some(code)), ExitClass::Config(_)),
                "{code}"
            );
        }
        for code in [5, 69] {
            assert!(
                matches!(classify(Some(code)), ExitClass::Operator(_)),
                "{code}"
            );
        }
        for code in [Some(0), Some(70), Some(137), None] {
            assert_eq!(classify(code), ExitClass::Transient, "{code:?}");
        }
    }

    #[test]
    fn gives_up_after_repeated_fast_config_failures() {
        let mut h = ExitHistory::default();
        assert_eq!(h.record(Some(3), QUICK), Decision::Restart);
        assert_eq!(h.record(Some(3), QUICK), Decision::Restart);
        match h.record(Some(3), QUICK) {
            Decision::GiveUp(reason) => {
                assert!(reason.contains("pipeline failed to load"), "{reason}");
                assert!(reason.contains("3 fast exits"), "{reason}");
            }
            other => panic!("expected give-up, got {other:?}"),
        }
    }

    #[test]
    fn phase_one_codes_give_up_too() {
        for code in [64, 69] {
            let mut h = ExitHistory::default();
            h.record(Some(code), QUICK);
            h.record(Some(code), QUICK);
            assert!(
                matches!(h.record(Some(code), QUICK), Decision::GiveUp(_)),
                "{code}"
            );
        }
    }

    #[test]
    fn a_different_status_restarts_the_count() {
        let mut h = ExitHistory::default();
        h.record(Some(3), QUICK);
        h.record(Some(3), QUICK);
        assert_eq!(h.record(Some(4), QUICK), Decision::Restart);
        assert_eq!(h.streak, 1);
    }

    #[test]
    fn a_slow_exit_resets_the_count() {
        // Ran for a while, then failed: the config was loadable, so a later
        // quick failure starts counting again from one.
        let mut h = ExitHistory::default();
        h.record(Some(4), QUICK);
        h.record(Some(4), QUICK);
        assert_eq!(h.record(Some(4), LONG), Decision::Restart);
        assert_eq!(h.streak, 0);
        assert_eq!(h.record(Some(4), QUICK), Decision::Restart);
    }

    #[test]
    fn crashes_and_internal_errors_never_give_up() {
        let mut h = ExitHistory::default();
        for _ in 0..10 {
            assert_eq!(h.record(Some(70), QUICK), Decision::Restart);
            assert_eq!(h.record(None, QUICK), Decision::Restart);
        }
    }
}
