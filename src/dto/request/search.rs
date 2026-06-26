//! Request DTOs for the natural-language / semantic event search endpoints.

use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Query parameters for `GET /api/v3/search`.
#[derive(Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct SearchQueryParams {
    /// The natural-language query.
    #[schema(example = "a person near the front door at night")]
    #[garde(length(min = 1, max = 512))]
    pub q: String,

    /// Restrict to a single monitor (still subject to the caller's ACL).
    #[schema(example = "1")]
    #[garde(range(min = 1, max = 1_000_000))]
    pub monitor_id: Option<u32>,

    /// Inclusive lower time bound, epoch seconds.
    #[schema(example = "1700000000")]
    #[garde(skip)]
    pub from: Option<i64>,

    /// Inclusive upper time bound, epoch seconds.
    #[schema(example = "1700003600")]
    #[garde(skip)]
    pub to: Option<i64>,

    /// Comma-separated object/class pre-filter (e.g. `person,car`).
    #[schema(example = "person,car")]
    #[garde(length(max = 256))]
    pub class: Option<String>,

    /// Number of results to return (1–50, default 10).
    #[schema(example = "10")]
    #[garde(range(min = 1, max = 50))]
    pub k: Option<usize>,
}

/// Query parameters for `GET /api/v3/events/{id}/similar`.
#[derive(Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct SimilarQueryParams {
    /// Number of results to return (1–50, default 10).
    #[schema(example = "10")]
    #[garde(range(min = 1, max = 50))]
    pub k: Option<usize>,
}
