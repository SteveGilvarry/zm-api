//! zm-api side of the zm-next worker integration.
//!
//! * [`detail`] — typed views over the EVENT `json_detail` JSON payloads.
//! * [`graph`] — validation for the stored processing plugin graph.
//! * [`ingest`] — maps decoded monitor EVENTs onto Events/Frames rows.
//! * [`pipeline`] — generates/composes a worker pipeline JSON from a monitor,
//!   its zones, and (when present) its stored processing graph.

pub mod detail;
pub mod graph;
pub mod ingest;
pub mod pipeline;

pub use ingest::EventIngestor;
pub use pipeline::generate_pipeline;
