//! Daemon controller module for managing ZoneMinder daemons.
//!
//! This module provides functionality to replace zmdc.pl and zmpkg.pl,
//! enabling process management for ZoneMinder daemons via both Unix socket
//! IPC (for legacy compatibility) and REST API endpoints.

pub mod backoff;
pub mod commands;
pub mod config;
pub mod daemons;
pub mod ipc;
pub mod manager;
pub mod process;
pub mod stats;

pub use config::DaemonConfig;
pub use manager::DaemonManager;
pub use process::{ManagedProcess, ProcessState};
