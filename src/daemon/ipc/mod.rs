//! IPC (Inter-Process Communication) module for the daemon controller.
//!
//! Provides Unix domain socket server for legacy compatibility with
//! zmdc.pl clients.

pub mod protocol;
pub mod socket;

pub use protocol::{DaemonCommand, DaemonResponse, ProcessStatus, SystemStats, SystemStatus};
pub use socket::DaemonSocketServer;
