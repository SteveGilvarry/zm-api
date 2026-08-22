//! Native replacements for ZoneMinder's Perl maintenance daemons.
//!
//! `zmstats.pl`, `zmaudit.pl` and `zmtelemetry.pl` are periodic housekeeping
//! jobs whose only state is the database, which makes them the cheapest of the
//! Perl daemons to absorb. Running them here removes three supervised processes
//! and puts their logging in the same journal as everything else.
//!
//! Each is independently switchable because the risk profiles differ sharply:
//! stats only writes bounded rows, telemetry only talks to the network, and the
//! audit deletes things.

pub mod audit;
pub mod stats;
pub mod telemetry;
