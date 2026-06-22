//! zm-api side of the zm-next worker integration.
//!
//! * [`detail`] — typed views over the EVENT `json_detail` JSON payloads.
//! * [`ingest`] — maps decoded monitor EVENTs onto Events/Frames rows.
//! * [`pipeline`] — generates a worker pipeline JSON from a monitor + its zones.

pub mod detail;
pub mod ingest;
pub mod pipeline;

pub use ingest::EventIngestor;
pub use pipeline::generate_pipeline;
