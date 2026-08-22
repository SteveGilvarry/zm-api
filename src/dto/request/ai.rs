//! Requests for the AI object-detection registry (datasets, models, classes).
//!
//! Field sets mirror ZoneMinder's own Options UI for these tables
//! (`web/skins/classic/views/_options_ai_*.php`), so a client written against
//! the legacy admin screens maps one-to-one onto this API.

use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entity::sea_orm_active_enums::Framework;

// --- Datasets --------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateAiDatasetRequest {
    #[garde(length(min = 1, max = 64))]
    pub name: String,
    #[garde(skip)]
    pub description: Option<String>,
    #[garde(inner(length(max = 32)))]
    pub version: Option<String>,
    /// Number of classes the dataset defines (COCO = 80).
    #[garde(range(min = 0))]
    pub num_classes: u32,
}

#[derive(Debug, Default, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateAiDatasetRequest {
    #[garde(inner(length(min = 1, max = 64)))]
    pub name: Option<String>,
    #[garde(skip)]
    pub description: Option<String>,
    #[garde(inner(length(max = 32)))]
    pub version: Option<String>,
    #[garde(skip)]
    pub num_classes: Option<u32>,
}

// --- Models ----------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateAiModelRequest {
    #[garde(length(min = 1, max = 64))]
    pub name: String,
    #[garde(skip)]
    pub description: Option<String>,
    /// Filesystem path to the model weights.
    #[garde(inner(length(max = 255)))]
    pub model_path: Option<String>,
    #[garde(skip)]
    pub framework: Option<Framework>,
    #[garde(inner(length(max = 32)))]
    pub version: Option<String>,
    /// Dataset whose classes this model predicts.
    #[garde(skip)]
    pub dataset_id: Option<u32>,
    #[garde(range(min = 0, max = 1))]
    pub enabled: Option<u8>,
}

#[derive(Debug, Default, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateAiModelRequest {
    #[garde(inner(length(min = 1, max = 64)))]
    pub name: Option<String>,
    #[garde(skip)]
    pub description: Option<String>,
    #[garde(inner(length(max = 255)))]
    pub model_path: Option<String>,
    #[garde(skip)]
    pub framework: Option<Framework>,
    #[garde(inner(length(max = 32)))]
    pub version: Option<String>,
    /// Set to re-point the model at another dataset. `null` clears it.
    #[garde(skip)]
    pub dataset_id: Option<Option<u32>>,
    #[garde(range(min = 0, max = 1))]
    pub enabled: Option<u8>,
}

// --- Object classes --------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateAiObjectClassRequest {
    #[garde(skip)]
    pub dataset_id: u32,
    #[garde(length(min = 1, max = 64))]
    pub class_name: String,
    /// Index this class occupies in the model's output vector.
    #[garde(range(min = 0))]
    pub class_index: u32,
    #[garde(skip)]
    pub description: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateAiObjectClassRequest {
    #[garde(skip)]
    pub dataset_id: Option<u32>,
    #[garde(inner(length(min = 1, max = 64)))]
    pub class_name: Option<String>,
    #[garde(skip)]
    pub class_index: Option<u32>,
    #[garde(skip)]
    pub description: Option<String>,
}

/// Filter for listing object classes.
#[derive(Debug, Default, Deserialize, Serialize, ToSchema, Validate)]
pub struct AiObjectClassQuery {
    /// Only return classes belonging to this dataset.
    #[garde(skip)]
    pub dataset_id: Option<u32>,
}
