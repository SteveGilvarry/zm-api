//! Responses for the AI object-detection registry.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::dto::PaginatedResponse;

/// Serialize a DB enum to its serde variant name, so the value a GET returns is
/// the value a POST/PATCH accepts (same approach as `MonitorResponse`).
fn enum_str<T: Serialize>(v: &T) -> String {
    match serde_json::to_value(v) {
        Ok(serde_json::Value::String(s)) => s,
        other => other.map(|o| o.to_string()).unwrap_or_default(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AiDatasetResponse {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub num_classes: u32,
}

impl From<&crate::entity::ai_datasets::Model> for AiDatasetResponse {
    fn from(m: &crate::entity::ai_datasets::Model) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
            description: m.description.clone(),
            version: m.version.clone(),
            num_classes: m.num_classes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AiModelResponse {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    pub model_path: Option<String>,
    /// One of TensorFlow, PyTorch, ONNX, OpenVINO, TensorRT, Other.
    pub framework: String,
    pub version: Option<String>,
    pub dataset_id: Option<u32>,
    /// Name of the linked dataset, when the listing joins it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_name: Option<String>,
    pub enabled: u8,
}

impl From<&crate::entity::ai_models::Model> for AiModelResponse {
    fn from(m: &crate::entity::ai_models::Model) -> Self {
        Self {
            id: m.id,
            name: m.name.clone(),
            description: m.description.clone(),
            model_path: m.model_path.clone(),
            framework: enum_str(&m.framework),
            version: m.version.clone(),
            dataset_id: m.dataset_id,
            dataset_name: None,
            enabled: m.enabled,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AiObjectClassResponse {
    pub id: u32,
    pub dataset_id: u32,
    pub class_name: String,
    pub class_index: u32,
    pub description: Option<String>,
}

impl From<&crate::entity::ai_object_classes::Model> for AiObjectClassResponse {
    fn from(m: &crate::entity::ai_object_classes::Model) -> Self {
        Self {
            id: m.id,
            dataset_id: m.dataset_id,
            class_name: m.class_name.clone(),
            class_index: m.class_index,
            description: m.description.clone(),
        }
    }
}

macro_rules! paginated {
    ($name:ident, $item:ty) => {
        #[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
        pub struct $name {
            pub items: Vec<$item>,
            pub total: u64,
            pub per_page: u64,
            pub current_page: u64,
            pub last_page: u64,
        }

        impl From<PaginatedResponse<$item>> for $name {
            fn from(r: PaginatedResponse<$item>) -> Self {
                Self {
                    items: r.items,
                    total: r.total,
                    per_page: r.per_page,
                    current_page: r.current_page,
                    last_page: r.last_page,
                }
            }
        }
    };
}

paginated!(PaginatedAiDatasetsResponse, AiDatasetResponse);
paginated!(PaginatedAiModelsResponse, AiModelResponse);
paginated!(PaginatedAiObjectClassesResponse, AiObjectClassResponse);
